use axum::{
    extract::Path,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use comrak::{ComrakOptions, markdown_to_html};
use serde::Deserialize;
use std::{fs, path::Path as StdPath};
use tera::{Context, Tera};

#[derive(Debug, Deserialize)]
struct FrontMatter {
    title: Option<String>,
    date: Option<String>,
}

pub async fn render_page(path: Option<Path<String>>) -> Html<String> {
    let md_input = match path {
        None => {
            // Blog Home
            fs::read_to_string("content/index.md").unwrap_or_default()
        }
        Some(ref path) => {
            if path.as_str().contains("..") {
                // TODO: 404, prevent traversal
                return Html("404".to_owned());
            }
            let file_name = format!("{}.md", path.as_str());
            let maybe_path = StdPath::new("content").join(file_name);
            match maybe_path.exists() {
                true => {
                    // Matching blog post
                    fs::read_to_string(format!("content/{}.md", path.as_str())).unwrap_or_default()
                }
                false => {
                    // TODO: 404
                    fs::read_to_string("content/index.md").unwrap_or_default()
                }
            }
        }
    };
    // Split front matter and markdown body
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

    let metadata: FrontMatter = toml::from_str(front_matter_str).unwrap();
    println!("{:?}", metadata);
    let html_content = markdown_to_html(&md_body, &ComrakOptions::default());

    // Load and render Tera template
    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    context.insert("title", &metadata.title);
    context.insert("content", &html_content);

    let rendered = tera.render("page.html", &context).unwrap();

    Html(rendered)
}

pub async fn ignore_favicon() -> Response {
    ().into_response()
    // TODO: Look into serving favicon
}
