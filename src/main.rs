use anyhow::Result;
use clap::Parser;
use firecrawl_cli::api::FirecrawlClient;
use firecrawl_cli::{cli::Cli, utils::*};

// Async main function that handles CLI commands and orchestrates the scraping/crawling process
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    // Parse command line arguments using clap
    let cli = Cli::parse();

    // Initialize the Firecrawl API client with the provided URL and API key
    let client = FirecrawlClient::new(&cli.api_url, cli.api_key.as_deref())?;

    // Handle different CLI commands: Scrape and Crawl
    match cli.command {
        // Handle the Scrape command for single page scraping
        firecrawl_cli::cli::Commands::Scrape { url, output_dir } => {
            println!("🔥 Scraping: {}", url);

            // Execute the scrape request to the API
            match client.scrape(&url).await {
                Ok(result) => {
                    // Display the scrape result summary
                    println!("{}", result);

                    // Save HTML content if available
                    if let Some(html) = result.html {
                        save_html(&output_dir, &url, &html, result.metadata.title.as_deref())
                            .await?;
                    }

                    // Save Markdown content if available
                    if let Some(markdown) = result.markdown {
                        save_markdown(
                            &output_dir,
                            &url,
                            &markdown,
                            result.metadata.title.as_deref(),
                        )
                        .await?;
                    }

                    // Save metadata as JSON if there's any metadata available
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
                    // Handle scraping errors and display user-friendly message
                    eprintln!("❌ Scrape failed: {}", e);
                    return Err(e);
                }
            }
        }
        // Handle the Crawl command for multi-page crawling
        firecrawl_cli::cli::Commands::Crawl {
            url,
            limit,
            output_dir,
        } => {
            println!("🕷️  Crawling: {} (limit: {:?})", url, limit);

            // Execute the crawl request to the API with specified page limit
            match client.crawl(&url, Some(limit)).await {
                Ok(results) => {
                    // Check if any pages were crawled
                    if results.is_empty() {
                        println!("⚠️  No pages were crawled");
                        return Ok(());
                    }

                    // Process each crawled page result
                    for (i, result) in results.iter().enumerate() {
                        // Save markdown content if available
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

                        // Save metadata as JSON if available
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

                        // Display progress for each processed page
                        let result_url = result.url.as_deref().unwrap_or(&url);
                        println!("✅ Processed {}/{}: {}", i + 1, results.len(), result_url);
                    }

                    // Display final crawl completion summary
                    println!("🎉 Crawling completed! Processed {} pages", results.len());
                }
                Err(e) => {
                    // Handle crawling errors and display user-friendly message
                    eprintln!("❌ Crawl failed: {}", e);
                    return Err(e);
                }
            }
        }
    }

    // Return success if all operations completed
    Ok(())
}
