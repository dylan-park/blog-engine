use axum::{extract::Path, response::Html};
use chrono::NaiveDate;
use comrak::{ComrakOptions, markdown_to_html};
use serde::Deserialize;
use std::{fs, path::Path as StdPath};
use tera::{Context, Tera};

#[derive(Debug, Deserialize)]
pub struct FrontMatter {
    title: Option<String>,
    date: Option<NaiveDate>,
    category: Option<String>,
}

pub async fn render_page(path: Option<Path<String>>) -> Html<String> {
    let md_input = match path {
        None => {
            // Blog Home
            "".to_owned()
        }
        Some(ref path) => {
            if path.as_str().contains("..") {
                // TODO: 404, prevent traversal
                return Html("404".to_owned());
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

    let tera = Tera::new("templates/**/*").unwrap();
    let mut context = Context::new();
    if md_input == "".to_owned() {
        // Blog Home
        context.insert("title", "Blog");
        let rendered = tera.render("blog.html", &context).unwrap();
        Html(rendered)
    } else if md_input == "404".to_owned() {
        // 404
        let rendered = tera.render("404.html", &context).unwrap();
        Html(rendered)
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

        Html(rendered)
    }
}

pub async fn parse_markdown_with_front_matter(
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

pub async fn format_date(input: Option<NaiveDate>) -> String {
    input
        .map(|d| d.format("%B %e, %Y").to_string())
        .unwrap_or_else(|| "Unknown date".to_string())
}
