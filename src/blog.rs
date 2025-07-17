use crate::{
    PageQuery,
    utils::{self, error::AppError, memory_manager},
};
use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Html,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::{fs, path::Path as StdPath};
use tera::{Context as TeraContext, Tera};
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub date: Option<NaiveDate>,
    pub categories: Option<Vec<String>>,
    pub summary: Option<String>,
    pub content_hash: Option<String>,
}

pub async fn render_page(
    path: Option<Path<String>>,
    Query(params): Query<PageQuery>,
) -> Result<(StatusCode, Html<String>), AppError> {
    let tera =
        Tera::new("templates/**/*").context("Failed to load templates from 'templates/**/*'")?;
    let mut context = TeraContext::new();

    let md_input = match path {
        None => {
            // Blog Home
            "".to_owned()
        }
        Some(ref path) => {
            if path.as_str().contains("..") {
                // 404 (Traversal prevention)
                return Ok((
                    StatusCode::NOT_FOUND,
                    Html(
                        tera.render("404.html", &context)
                            .context("Error rendering template: '404.html'")?,
                    ),
                ));
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
        let recent_posts = render_posts(
            params.clone(),
            memory_manager::get_all_posts_sorted_by_date()
                .await
                .context("Unable to get all posts sorted by date")?,
        )
        .await
        .context("Unable to render recent posts")?;
        context.insert("title", "Blog");
        context.insert("posts", &recent_posts.0);
        context.insert("pagination", &recent_posts.1);
        let rendered = tera
            .render("blog.html", &context)
            .context("Error rendering template: 'blog.html'")?;
        info!("Served: Blog Home | Page {}", params.page.unwrap_or(1));
        Ok((StatusCode::OK, Html(rendered)))
    } else if md_input == "404" {
        // 404
        let rendered = tera
            .render("404.html", &context)
            .context("Error rendering template: '404.html'")?;
        info!(
            "Served: 404 | {}",
            path.as_ref()
                .map(|p| p.0.as_str())
                .unwrap_or("<unknown path>")
        );
        Ok((StatusCode::NOT_FOUND, Html(rendered)))
    } else {
        // Matching blog post
        let parsed_input = utils::parse_markdown_with_frontmatter(md_input)
            .await
            .context("Unable to parse markdown with frontmatter")?;

        let metadata = parsed_input.0;
        let html_content = parsed_input.1;
        context.insert("title", &metadata.title);
        context.insert(
            "category",
            &metadata.categories.context("Unable to get post category")?[0],
        );
        context.insert("content", &html_content);

        let formatted_date = utils::format_date(metadata.date).await;
        context.insert("date", &formatted_date.context("Unable to format date")?);

        let rendered = tera
            .render("post.html", &context)
            .context("Error rendering template: 'post.html'")?;
        info!(
            "Served: Blog Post | {}",
            &metadata.title.as_deref().unwrap_or("<unknown title>")
        );
        Ok((StatusCode::OK, Html(rendered)))
    }
}

pub async fn render_category_page(
    Query(params): Query<PageQuery>,
) -> Result<(StatusCode, Html<String>), AppError> {
    let tera =
        Tera::new("templates/**/*").context("Failed to load templates from 'templates/**/*'")?;
    let mut context = TeraContext::new();
    let category = params.clone().category.unwrap_or("".to_string());
    let page = params.page.unwrap_or(1);
    let recent_posts = render_posts(
        params,
        memory_manager::get_posts_by_category(&category)
            .await
            .context("Failed to get posts by category")?,
    )
    .await
    .context("Unable to render recent posts")?;
    context.insert("title", "Blog");
    context.insert("posts", &recent_posts.0);
    context.insert("pagination", &recent_posts.1);
    context.insert("category", &category);
    let rendered = tera
        .render("blog.html", &context)
        .context("Error rendering template: 'blog.html'")?;
    info!("Served: Blog Category | {category} | Page {}", page);
    Ok((StatusCode::OK, Html(rendered)))
}

async fn render_posts(params: PageQuery, files_input: Vec<String>) -> Result<(String, String)> {
    let total_files = files_input.len();
    let page = params.page.unwrap_or(1);
    let category = params.category.unwrap_or("".to_string());
    // Query String pagination
    let files = files_input
        .chunks(3)
        .nth(page - 1)
        .map(|chunk| chunk.to_vec())
        .unwrap_or_default();

    let mut posts_output = "".to_owned();

    for file in &files {
        let md_input = fs::read_to_string(format!("posts/{file}.md")).unwrap_or_default();
        let parsed_input = utils::parse_markdown_with_frontmatter(md_input)
            .await
            .context("Unable to parse markdown with frontmatter")?;
        posts_output = format!(
            "{}{}",
            posts_output,
            generate_blog_post_card(
                utils::format_date(parsed_input.0.date)
                    .await
                    .context("Unable to format date")?,
                parsed_input
                    .0
                    .categories
                    .context("Unable to get post category")?[0]
                    .clone(),
                file.to_owned(),
                parsed_input.0.title.context("Unable to get post title")?,
                memory_manager::get_post_summary(file.to_string())
                    .await
                    .context("Unable to get post summary")?,
            )
            .await
            .context("Error generating blog post card")?
        );
    }

    let pagination_output = generate_pagination(total_files, page, category)
        .await
        .context("Error generating pagination")?;

    Ok((posts_output, pagination_output))
}

async fn generate_blog_post_card(
    date: String,
    category: String,
    path: String,
    title: String,
    content: String,
) -> Result<String> {
    let tera =
        Tera::new("templates/**/*").context("Failed to load templates from 'templates/**/*'")?;
    let mut context = TeraContext::new();
    context.insert("date", &date);
    context.insert("category", &category);
    context.insert("path", &path);
    context.insert("title", &title);
    context.insert("content", &content);

    tera.render("blog-post-card.html", &context)
        .context("Error rendering template: 'blog-post-card.html'")
}

async fn generate_pagination(total_files: usize, page: usize, category: String) -> Result<String> {
    let tera =
        Tera::new("templates/**/*").context("Failed to load templates from 'templates/**/*'")?;
    let mut context = TeraContext::new();

    if !category.is_empty() {
        context.insert("category", &category);
    }

    if total_files > 3 && (page - 1) * 3 < total_files {
        let total_pages = total_files.div_ceil(3);

        context.insert("page", &page);
        context.insert("total_pages", &total_pages);

        Ok(tera
            .render("pagination.html", &context)
            .context("Error rendering template: 'pagination.html'")?)
    } else {
        Ok(String::new())
    }
}
