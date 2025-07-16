use crate::{blog::FrontMatter, utils::general};
use anyhow::{Context, Result};
use futures::future::join_all;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::{cmp::Reverse, collections::HashMap, path::Path, time::Duration};
use tokio::{
    fs, pin,
    sync::{RwLock, mpsc},
    time::sleep,
};
use tracing::{error, info, warn};

struct FrontmatterIndex {
    posts: HashMap<String, FrontMatter>,
}

static FRONTMATTER_INDEX: Lazy<RwLock<FrontmatterIndex>> = Lazy::new(|| {
    RwLock::new(FrontmatterIndex {
        posts: HashMap::new(),
    })
});

pub async fn setup_file_watcher() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(100);
    let tx_clone = tx.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res
                && matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                )
            {
                let _ = tx_clone.try_send(event);
            }
        },
        Config::default(),
    )?;

    watcher.watch(Path::new("posts/"), RecursiveMode::Recursive)?;

    tokio::spawn(async move {
        // Keep watcher alive for the task lifetime
        let _watcher = watcher;

        while let Some(_event) = rx.recv().await {
            // Consume any additional events that arrive during debounce period
            let debounce_timer = sleep(Duration::from_millis(100));
            pin!(debounce_timer);

            loop {
                tokio::select! {
                    _ = &mut debounce_timer => {
                        // Debounce period elapsed, process the rebuild
                        break;
                    }
                    additional_event = rx.recv() => {
                        match additional_event {
                            Some(_) => {
                                // Reset the debounce timer for additional events
                                debounce_timer.set(sleep(Duration::from_millis(100)));
                            }
                            None => return, // Channel closed
                        }
                    }
                }
            }

            // Rebuild index when posts directory changes
            if let Err(err) = build_frontmatter_index().await {
                error!("Error rebuilding index: {:?}", err);
            }
        }

        warn!("File watcher task ended unexpectedly");
    });

    info!("File watcher spawned on: posts/");
    Ok(())
}

pub async fn build_frontmatter_index() -> Result<()> {
    let file_names = general::get_all_posts()
        .await
        .context("Unable to get all posts")?;

    // Process files concurrently
    let tasks: Vec<_> = file_names
        .into_iter()
        .map(|file_name| async move {
            let file_path = format!("posts/{file_name}.md");

            let content = fs::read_to_string(&file_path)
                .await
                .with_context(|| format!("Failed to read file: {file_path}"))?;

            let frontmatter = general::parse_markdown_with_frontmatter(content)
                .await
                .with_context(|| format!("Failed to parse frontmatter for: {file_name}"))?
                .0;

            Ok::<_, anyhow::Error>((file_name, frontmatter))
        })
        .collect();

    // Wait for all tasks to complete
    let results = join_all(tasks).await;

    // Build the index from successful results
    let mut posts = HashMap::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok((file_name, frontmatter)) => {
                posts.insert(file_name, frontmatter);
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    // Log errors but don't fail the entire rebuild
    for error in &errors {
        error!("Error processing file: {:#}", error);
    }

    // Update the index with successful results
    let posts_count = posts.len();
    let index = FrontmatterIndex { posts };
    *FRONTMATTER_INDEX.write().await = index;

    info!(
        "Frontmatter Index rebuilt with {} posts ({} errors)",
        posts_count,
        errors.len()
    );

    if posts_count == 0 && !errors.is_empty() {
        anyhow::bail!("Failed to process any posts");
    }

    Ok(())
}

async fn get_posts_with_filter<F>(filter_fn: F) -> Result<Vec<String>>
where
    F: Fn(&str, &FrontMatter) -> bool,
{
    let index = FRONTMATTER_INDEX.read().await;

    let mut posts_with_dates: Vec<_> = index
        .posts
        .iter()
        .filter_map(|(filename, fm)| {
            if filter_fn(filename, fm) {
                fm.date.map(|date| (Reverse(date), filename.as_str()))
            } else {
                None
            }
        })
        .collect();

    posts_with_dates.sort_unstable_by_key(|(rev_date, _)| *rev_date);

    Ok(posts_with_dates
        .into_iter()
        .map(|(_, filename)| filename.to_string())
        .collect())
}

pub async fn get_all_posts_sorted_by_date() -> Result<Vec<String>> {
    get_posts_with_filter(|_, _| true).await
}

pub async fn get_posts_by_category(category: &str) -> Result<Vec<String>> {
    if category.is_empty() {
        return Ok(Vec::new());
    }

    get_posts_with_filter(|_, fm| {
        fm.categories
            .as_ref()
            .is_some_and(|cats| cats.iter().any(|c| c.eq_ignore_ascii_case(category)))
    })
    .await
}
