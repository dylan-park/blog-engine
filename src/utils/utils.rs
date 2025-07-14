use crate::blog::FrontMatter;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use comrak::{ComrakOptions, markdown_to_html};
use std::fs;

pub async fn parse_markdown_with_frontmatter(md_input: String) -> Result<(FrontMatter, String)> {
    let (frontmatter_str, md_body) = if md_input.starts_with("+++") {
        let parts: Vec<&str> = md_input.splitn(3, "+++").collect();
        if parts.len() == 3 {
            (parts[1].trim(), parts[2].trim())
        } else {
            ("", md_input.as_str())
        }
    } else {
        ("", md_input.as_str())
    };

    let metadata: FrontMatter = toml::from_str(frontmatter_str)?;
    let html_content = markdown_to_html(md_body, &ComrakOptions::default());

    Ok((metadata, html_content))
}

pub async fn truncate_html_text(html: &str, max_length: usize) -> Result<String> {
    let text = strip_html_tags(html)
        .await
        .context("Unable to strip html tags")?;

    if text.len() <= max_length {
        return Ok(text);
    }

    let truncated = find_word_boundary(&text, max_length)
        .await
        .context("Unable to find word boundary")?;

    if truncated.len() < text.len() {
        Ok(format!("{truncated}..."))
    } else {
        Ok(truncated)
    }
}

pub async fn get_all_posts() -> Result<Vec<String>> {
    Ok(fs::read_dir("posts")
        .context("Unable to read directory 'posts'")?
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
        .collect())
}

async fn strip_html_tags(html: &str) -> Result<String> {
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

    Ok(result.split_whitespace().collect::<Vec<_>>().join(" "))
}

async fn find_word_boundary(text: &str, max_length: usize) -> Result<String> {
    if text.len() <= max_length {
        return Ok(text.to_string());
    }

    let mut end_pos = max_length;
    let chars: Vec<char> = text.chars().collect();

    if end_pos < chars.len() && !chars[end_pos].is_whitespace() {
        while end_pos < chars.len() && !chars[end_pos].is_whitespace() {
            end_pos += 1;
        }
    }

    Ok(chars
        .iter()
        .take(end_pos)
        .collect::<String>()
        .trim_end()
        .to_string())
}

pub async fn format_date(input: Option<NaiveDate>) -> Result<String> {
    Ok(input
        .map(|d| d.format("%B %e, %Y").to_string())
        .unwrap_or_else(|| "Unknown date".to_string()))
}
