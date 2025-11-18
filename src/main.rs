use anyhow::Result;
use clap::Parser;
use firecrawl_cli::api::FirecrawlClient;
use firecrawl_cli::{cli::Cli, utils::*};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = FirecrawlClient::new(&cli.api_url, cli.api_key.as_deref())?;

    match cli.command {
        firecrawl_cli::cli::Commands::Scrape { url, output_dir } => {
            println!("🔥 Scraping: {}", url);

            match client.scrape(&url).await {
                Ok(result) => {
                    println!("{}", result);

                    if let Some(html) = result.html {
                        save_html(&output_dir, &url, &html, result.metadata.title.as_deref())
                            .await?;
                    }

                    if let Some(markdown) = result.markdown {
                        save_markdown(
                            &output_dir,
                            &url,
                            &markdown,
                            result.metadata.title.as_deref(),
                        )
                        .await?;
                    }

                    if !result.metadata.extra.is_empty() || result.metadata.title.is_some() {
                        let metadata = serde_json::json!({
                            "title": result.metadata.title,
                            "description": result.metadata.description,
                            "language": result.metadata.language,
                            "source_url": result.metadata.source_url,
                            "extra": result.metadata.extra
                        });
                        save_json(
                            &output_dir,
                            &url,
                            &metadata,
                            result.metadata.title.as_deref(),
                        )
                        .await?;
                    }

                    println!("✅ Scrape completed successfully!");
                }
                Err(e) => {
                    eprintln!("❌ Scrape failed: {}", e);
                    return Err(e);
                }
            }
        }
        firecrawl_cli::cli::Commands::Crawl {
            url,
            limit,
            output_dir,
        } => {
            println!("🕷️  Crawling: {} (limit: {:?})", url, limit);

            match client.crawl(&url, Some(limit)).await {
                Ok(results) => {
                    if results.is_empty() {
                        println!("⚠️  No pages were crawled");
                        return Ok(());
                    }

                    for (i, result) in results.iter().enumerate() {
                        if let Some(markdown) = &result.markdown {
                            let result_url = result.url.as_deref().unwrap_or(&url);
                            save_markdown(
                                &output_dir,
                                result_url,
                                markdown,
                                result.metadata.title.as_deref(),
                            )
                            .await?;
                        }

                        if !result.metadata.extra.is_empty() || result.metadata.title.is_some() {
                            let result_url = result.url.as_deref().unwrap_or(&url);
                            let metadata = serde_json::json!({
                                "title": result.metadata.title,
                                "description": result.metadata.description,
                                "language": result.metadata.language,
                                "source_url": result.metadata.source_url,
                                "extra": result.metadata.extra
                            });
                            save_json(
                                &output_dir,
                                result_url,
                                &metadata,
                                result.metadata.title.as_deref(),
                            )
                            .await?;
                        }

                        let result_url = result.url.as_deref().unwrap_or(&url);
                        println!("✅ Processed {}/{}: {}", i + 1, results.len(), result_url);
                    }

                    println!("🎉 Crawling completed! Processed {} pages", results.len());
                }
                Err(e) => {
                    eprintln!("❌ Crawl failed: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}
