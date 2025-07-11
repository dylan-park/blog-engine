use crate::{
    PageQuery,
    utils::utils::{format_date, parse_markdown_with_front_matter, truncate_html_text},
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Html,
};
use chrono::NaiveDate;
use log::info;
use serde::Deserialize;
use std::{fs, path::Path as StdPath};
use tera::{Context, Tera};
#[derive(Debug, Deserialize)]
pub struct FrontMatter {
    title: Option<String>,
    date: Option<NaiveDate>,
    category: Option<String>,
}

pub async fn render_page(
    path: Option<Path<String>>,
    Query(params): Query<PageQuery>,
) -> (StatusCode, Html<String>) {
    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();

    let md_input = match path {
        None => {
            // Blog Home
            "".to_owned()
        }
        Some(ref path) => {
            if path.as_str().contains("..") {
                // 404 (Traversal prevention)
                return (
                    StatusCode::NOT_FOUND,
                    Html(tera.render("404.html", &context).unwrap()),
                );
            }
            let file_name = format!("{}.md", path.as_str());
            let maybe_path = StdPath::new("posts").join(file_name);
            match maybe_path.exists() {
                true => {
                    // Matching blog post
                    fs::read_to_string(format!("posts/{}.md", path.as_str())).unwrap_or_default()
                }
                false => "404".to_owned(),
            }
        }
    };

    if md_input.is_empty() {
        // Blog Home
        let recent_posts = render_recent_posts(params.page.unwrap_or(1)).await;
        context.insert("title", "Blog");
        context.insert("posts", &recent_posts.0);
        context.insert("pagination", &recent_posts.1);
        let rendered = tera.render("blog.html", &context).unwrap();
        info!("Served: Blog Home | Page {}", params.page.unwrap_or(1));
        (StatusCode::OK, Html(rendered))
    } else if md_input == "404" {
        // 404
        let rendered = tera.render("404.html", &context).unwrap();
        info!("Served: 404 | {}", path.unwrap().0);
        (StatusCode::NOT_FOUND, Html(rendered))
    } else {
        // Matching blog post
        let parsed_input = parse_markdown_with_front_matter(md_input).await.unwrap();

        let metadata = parsed_input.0;
        let html_content = parsed_input.1;
        context.insert("title", &metadata.title);
        context.insert("category", &metadata.category);
        context.insert("content", &html_content);

        let formatted_date = format_date(metadata.date).await;
        context.insert("date", &formatted_date);

        let rendered = tera.render("post.html", &context).unwrap();
        info!("Served: Blog Post | {}", &metadata.title.unwrap());
        (StatusCode::OK, Html(rendered))
    }
}

async fn render_recent_posts(page: usize) -> (String, String) {
    // Get all files from posts dir
    let mut files: Vec<String> = fs::read_dir("posts")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    // Sort descending
    files.sort_by(|a, b| b.cmp(a));
    let total_files = files.len();
    // Query String pagination
    files = files
        .chunks(3)
        .nth(page - 1)
        .map(|chunk| chunk.to_vec())
        .unwrap_or_default();

    let mut posts_output = "".to_owned();

    for file in &files {
        let md_input = fs::read_to_string(format!("posts/{file}.md")).unwrap_or_default();
        let parsed_input = parse_markdown_with_front_matter(md_input).await.unwrap();
        posts_output = format!(
            "{}{}",
            posts_output,
            generate_blog_post_card(
                format_date(parsed_input.0.date).await,
                parsed_input.0.category.unwrap(),
                file.to_owned(),
                parsed_input.0.title.unwrap(),
                truncate_html_text(parsed_input.1.as_str(), 240).await,
            )
            .await
        );
    }

    let pagination_output = generate_pagination(total_files, page).await;

    (posts_output, pagination_output)
}

async fn generate_blog_post_card(
    date: String,
    category: String,
    path: String,
    title: String,
    content: String,
) -> String {
    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("date", &date);
    context.insert("category", &category);
    context.insert("path", &path);
    context.insert("title", &title);
    context.insert("content", &content);

    tera.render("blog-post-card.html", &context).unwrap()
}

async fn generate_pagination(total_files: usize, page: usize) -> String {
    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();

    if total_files > 3 && (page - 1) * 3 < total_files {
        let total_pages = total_files.div_ceil(3);

        context.insert("page", &page);
        context.insert("total_pages", &total_pages);

        tera.render("pagination.html", &context).unwrap()
    } else {
        String::new()
    }
}
