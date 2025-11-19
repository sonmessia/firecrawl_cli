use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

// Save markdown content to a file with metadata header
pub async fn save_markdown(
    dir: &PathBuf,
    url: &str,
    content: &str,
    title: Option<&str>,
) -> Result<()> {
    // Create output directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
    }

    // Generate a safe filename from title or URL
    let raw_name = title.unwrap_or(url);
    let filename = format!("{}.md", slug::slugify(raw_name));
    let path = dir.join(filename);

    // Create file content with YAML frontmatter containing the source URL
    let file_content = format!("---\nurl: {}\n---\n\n{}", url, content);

    // Write the content to file
    fs::write(&path, file_content).await?;
    println!("💾 Saved markdown: {:?}", path);
    Ok(())
}

// Save HTML content to a file with metadata header
pub async fn save_html(dir: &PathBuf, url: &str, content: &str, title: Option<&str>) -> Result<()> {
    // Create output directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
    }

    // Generate a safe filename from title or URL
    let raw_name = title.unwrap_or(url);
    let filename = format!("{}.html", slug::slugify(raw_name));
    let path = dir.join(filename);

    // Create file content with YAML frontmatter containing the source URL
    let file_content = format!("---\nurl: {}\n---\n\n{}", url, content);

    // Write the content to file
    fs::write(&path, file_content).await?;
    println!("💾 Saved HTML: {:?}", path);
    Ok(())
}

// Save JSON metadata to a file
pub async fn save_json(
    dir: &PathBuf,
    url: &str,
    data: &serde_json::Value,
    title: Option<&str>,
) -> Result<()> {
    // Create output directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
    }

    // Generate a safe filename from title or URL
    let raw_name = title.unwrap_or(url);
    let filename = format!("{}.json", slug::slugify(raw_name));
    let path = dir.join(filename);

    // Convert JSON data to pretty-printed string
    let file_content = serde_json::to_string_pretty(data)?;
    fs::write(&path, file_content).await?;
    println!("💾 Saved JSON: {:?}", path);
    Ok(())
}
