use std::error::Error;
mod foundry_service;
use foundry_service::FoundryService;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let foundry_service = FoundryService::new().await?;
    //let io = (tokio::io::stdin(), tokio::io::stdout());

    //serve_server(foundry_service, io).await?;
    //Ok(())

    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::WARN)
    .init();

    tracing::info!("Starting MCP server");

    // Create an instance of our counter router
    let service = foundry_service.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())

} 