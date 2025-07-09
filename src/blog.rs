use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Html,
};
use chrono::NaiveDate;
use comrak::{ComrakOptions, markdown_to_html};
use log::info;
use serde::Deserialize;
use std::{fs, path::Path as StdPath};
use tera::{Context, Tera};

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    page: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct FrontMatter {
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
        info!("Served: Blog Home");
        (StatusCode::OK, Html(rendered))
    } else if md_input == "404" {
        // 404
        let rendered = tera.render("404.html", &context).unwrap();
        info!("Served: 404: {}", path.unwrap().0);
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
        info!("Served: Blog Post: {}", &metadata.title.unwrap());
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

    // Pagination link logic
    let pagination_output = if total_files > 3 && (page - 1) * 3 < total_files {
        let total_pages = total_files.div_ceil(3);

        match page {
            1 => format!(
                "<a class=\"pagination-btn next-btn\" href=\"?page=2\"><span>Next</span></a><span class=\"page-info\">Page {page} of {total_pages}</span>"
            ),
            p if p == total_pages => format!(
                "<a class=\"pagination-btn prev-btn\" href=\"?page={}\"><span>Previous</span></a><span class=\"page-info\">Page {} of {}</span>",
                page - 1,
                page,
                total_pages
            ),
            _ => format!(
                "<a class=\"pagination-btn prev-btn\" href=\"?page={}\"><span>Previous</span></a><span class=\"page-info\">Page {} of {}</span><a class=\"pagination-btn next-btn\" href=\"?page={}\"><span>Next</span></a>",
                page - 1,
                page,
                total_pages,
                page + 1
            ),
        }
    } else {
        String::new()
    };

    (posts_output, pagination_output)
}

async fn truncate_html_text(html: &str, max_length: usize) -> String {
    let text = strip_html_tags(html).await;

    if text.len() <= max_length {
        return text;
    }

    let truncated = find_word_boundary(&text, max_length).await;

    if truncated.len() < text.len() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

async fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;
    let chars = html.chars().peekable();

    for ch in chars {
        match ch {
            '<' => {
                inside_tag = true;
            }
            '>' => {
                inside_tag = false;
            }
            _ => {
                if !inside_tag {
                    result.push(ch);
                }
            }
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

async fn find_word_boundary(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }

    let mut end_pos = max_length;
    let chars: Vec<char> = text.chars().collect();

    if end_pos < chars.len() && !chars[end_pos].is_whitespace() {
        while end_pos < chars.len() && !chars[end_pos].is_whitespace() {
            end_pos += 1;
        }
    }

    chars
        .iter()
        .take(end_pos)
        .collect::<String>()
        .trim_end()
        .to_string()
}

async fn generate_blog_post_card(
    date: String,
    category: String,
    path: String,
    title: String,
    content: String,
) -> String {
    format!(
        r#"
        <article class="blog-post-card">
            <div class="post-meta">
                <span class="post-date">{date}</span>
                <span class="post-category">{category}</span>
            </div>
            <h3>
                <a href="{path}">{title}</a>
            </h3>
            <p>{content}</p>
            <a href="{path}" class="read-more">Read More</a>
        </article>
        "#
    )
}

async fn parse_markdown_with_front_matter(
    md_input: String,
) -> Result<(FrontMatter, String), Box<dyn std::error::Error>> {
    let (front_matter_str, md_body) = if md_input.starts_with("+++") {
        let parts: Vec<&str> = md_input.splitn(3, "+++").collect();
        if parts.len() == 3 {
            (parts[1].trim(), parts[2].trim())
        } else {
            ("", md_input.as_str())
        }
    } else {
        ("", md_input.as_str())
    };

    let metadata: FrontMatter = toml::from_str(front_matter_str)?;
    let html_content = markdown_to_html(md_body, &ComrakOptions::default());

    Ok((metadata, html_content))
}

async fn format_date(input: Option<NaiveDate>) -> String {
    input
        .map(|d| d.format("%B %e, %Y").to_string())
        .unwrap_or_else(|| "Unknown date".to_string())
}

pub async fn health_check() -> (StatusCode, Html<String>) {
    (StatusCode::OK, Html("OK".to_owned()))
}
