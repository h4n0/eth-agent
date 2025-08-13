use std::env;
use std::collections::HashMap;
use std::io::{self, Write};
use anyhow::Result;
use tracing::{error, info, Level};
use tracing_subscriber;

// Import modules
mod types;
mod mcp_client;
mod agent;
mod tools;


use types::*;
use agent::EthAgent;
use rig::providers::anthropic;

const ANTHROPIC_MODEL: &str = "claude-3-5-haiku-20241022";
const EVALUATION_THRESHOLD: u32 = 70;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    info!("Starting ETH Agent with MCP-based Foundry integration");

    let brave_search_api_key = env::var("BRAVE_SEARCH_API_KEY").expect("BRAVE_SEARCH_API_KEY must be set");

    // Create ETH Agent
    //let mut agent = EthAgent::<openai::Client>::new(api_key, Some(10))?;
    let mut agent = EthAgent::<anthropic::Client>::new(&brave_search_api_key, ANTHROPIC_MODEL, ANTHROPIC_MODEL, ANTHROPIC_MODEL, EVALUATION_THRESHOLD)?;

    // Initialize context
    let mut context = HashMap::new();
    context.insert("network".to_string(), serde_json::json!("foundry local"));

    println!("🤖 ETH Agent CLI REPL");
    println!("Type 'help' for available prompts, 'quit' to exit");
    println!("Network: local foundry");
    println!();

    // CLI REPL loop
    loop {
        // Print prompt
        print!("agent> ");
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Handle special prompts
        match input.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("Goodbye! 👋");
                break;
            }
            "help" | "h" => {
                print_help();
                continue;
            }
            "clear" | "cls" => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                continue;
            }
            "" => continue, // Empty input, continue to next iteration
            _ => {}
        }

        // Process the prompt
        let prompt = UserPrompt {
            id: uuid::Uuid::new_v4().to_string(),
            natural_language: input.to_string(),
            timestamp: chrono::Utc::now(),
            context: context.clone(),
        };

        info!("Processing prompt: {}", prompt.natural_language);

        match agent.run(prompt).await {
            Ok(result) => {
                println!("✅ Prompt executed successfully!");
                println!("🎯 Result: {}", result.result);

                if let Some(error) = result.error_message {
                    println!("⚠️  Execution completed with error: {}", error);
                }
            }
            Err(e) => {
                println!("❌ Failed to process prompt: {}", e);
                error!("Failed to process prompt: {}", e);
            }
        }
        println!(); // Add spacing between prompts
    }

    info!("Shutting down ETH Agent");
    Ok(())
}

fn print_help() {
    println!("\n📚 Available Commands:");
    println!("  help, h          - Show this help message");
    println!("  quit, exit, q    - Exit the REPL");
    println!("  clear, cls       - Clear the screen");
    println!("\n💡 Example Prompts:");
    println!("  Transfer 0.1 ETH to 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6");
    println!("  Deploy a simple contract");
    println!("  Call function 'mint' on contract 0x123...");
    println!("  Read balance of 0x456...");
    println!("\n🔧 Supported Actions (Planning Only):");
    println!("  • Transfer ETH");
    println!("  • Deploy contracts");
    println!("  • Call contract functions");
    println!("  • Read contract state");
    println!("  • Batch operations");
    println!("\n💬 Natural Language Prompts:");
    println!("  Type any request in natural language and the agent will plan and evaluate it.");
    println!("  Note: Actions are not executed in independent mode.");
    println!();
}