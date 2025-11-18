// src/utils.rs
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

pub async fn save_markdown(
    dir: &PathBuf,
    url: &str,
    content: &str,
    title: Option<&str>,
) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
    }

    // Tạo tên file safe
    let raw_name = title.unwrap_or(url);
    let filename = format!("{}.md", slug::slugify(raw_name));
    let path = dir.join(filename);

    let file_content = format!("---\nurl: {}\n---\n\n{}", url, content);

    fs::write(&path, file_content).await?;
    println!("💾 Đã lưu: {:?}", path);
    Ok(())
}

pub async fn save_html(dir: &PathBuf, url: &str, content: &str, title: Option<&str>) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
    }

    // Tạo tên file safe
    let raw_name = title.unwrap_or(url);
    let filename = format!("{}.html", slug::slugify(raw_name));
    let path = dir.join(filename);

    let file_content = format!("---\nurl: {}\n---\n\n{}", url, content);

    fs::write(&path, file_content).await?;
    println!("💾 Đã lưu raw html: {:?}", path);
    Ok(())
}

pub async fn save_json(
    dir: &PathBuf,
    url: &str,
    data: &serde_json::Value,
    title: Option<&str>,
) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
    }

    // Xử lý logic save file ở đây
    let raw_name = title.unwrap_or(url);
    let filename = format!("{}.json", slug::slugify(raw_name));
    let path = dir.join(filename);

    let file_content = serde_json::to_string_pretty(data)?;
    fs::write(&path, file_content).await?;
    println!("💾 Đã lưu JSON: {:?}", path);
    Ok(())
}
