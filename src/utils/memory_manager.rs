use crate::{
    blog::FrontMatter,
    utils::utils::{get_all_posts, parse_markdown_with_frontmatter},
};
use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::sync::RwLock;
use tracing::error;

struct FrontmatterIndex {
    posts: HashMap<String, FrontMatter>,
}

static FRONTMATTER_INDEX: Lazy<RwLock<FrontmatterIndex>> = Lazy::new(|| {
    RwLock::new(FrontmatterIndex {
        posts: HashMap::new(),
    })
});

pub async fn setup_file_watcher() -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) {
                    let _ = tx.blocking_send(event);
                }
            }
        },
        Config::default(),
    )?;

    watcher.watch(Path::new("posts/"), RecursiveMode::Recursive)?;

    tokio::spawn(async move {
        while let Some(_event) = rx.recv().await {
            // Rebuild index when posts directory changes
            if let Err(err) = build_frontmatter_index().await {
                error!("Error rebuilding index: {:?}", err);
            }
        }
    });

    // Keep watcher alive
    std::mem::forget(watcher);

    Ok(())
}

pub async fn build_frontmatter_index() -> Result<()> {
    let mut index = FrontmatterIndex {
        posts: HashMap::new(),
    };

    // Read all posts once at startup
    for file_name in get_all_posts().await.context("Unable to get all posts")? {
        let frontmatter = parse_markdown_with_frontmatter(
            fs::read_to_string(format!("posts/{}.md", file_name.as_str())).unwrap_or_default(),
        )
        .await
        .context("Unable to parse markdown with frontmatter")?
        .0;

        index.posts.insert(file_name.clone(), frontmatter);
    }

    *FRONTMATTER_INDEX.write().await = index;
    Ok(())
}

pub async fn get_posts_by_category(category: &str) -> Result<Vec<String>> {
    let index = FRONTMATTER_INDEX.read().await;

    let mut matching_posts: Vec<_> = index
        .posts
        .iter()
        .filter(|(_, fm)| {
            fm.categories
                .as_ref()
                .map_or(false, |cats| cats.iter().any(|c| c == category))
        })
        .filter_map(|(filename, fm)| fm.date.map(|date| (Reverse(date), filename)))
        .collect();

    // Sort by date (descending )
    matching_posts.sort_by_key(|(rev_date, _)| *rev_date);

    Ok(matching_posts
        .into_iter()
        .map(|(_, filename)| filename.to_string())
        .collect())
}

pub async fn get_all_posts_sorted_by_date() -> Result<Vec<String>> {
    let index = FRONTMATTER_INDEX.read().await;
    let mut files_with_dates: Vec<_> = index
        .posts
        .iter()
        .filter_map(|(filename, fm)| fm.date.as_ref().map(|date| (Reverse(*date), filename)))
        .collect();

    // Sort by date (descending )
    files_with_dates.sort_by_key(|(rev_date, _)| *rev_date);

    // To get filenames only:
    let sorted_filenames: Vec<String> = files_with_dates
        .into_iter()
        .map(|(_, filename)| filename.to_string())
        .collect();
    Ok(sorted_filenames)
}
