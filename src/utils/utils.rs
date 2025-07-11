use crate::blog::FrontMatter;
use chrono::NaiveDate;
use comrak::{ComrakOptions, markdown_to_html};

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

pub async fn truncate_html_text(html: &str, max_length: usize) -> String {
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

pub async fn format_date(input: Option<NaiveDate>) -> String {
    input
        .map(|d| d.format("%B %e, %Y").to_string())
        .unwrap_or_else(|| "Unknown date".to_string())
}
