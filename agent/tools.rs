use serde::Deserialize;
use serde_json::json;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use rig::{
    completion::ToolDefinition,
    tool::Tool,
};
use crate::mcp_client::FoundryMcpClient;

// Error types for different tool operations
#[derive(Debug)]
pub enum ToolError {
    McpError(anyhow::Error),
    SerializationError(serde_json::Error),
    InvalidAddress(String),
    InvalidTransactionParams(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::McpError(e) => write!(f, "MCP client error: {}", e),
            ToolError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ToolError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            ToolError::InvalidTransactionParams(params) => write!(f, "Invalid transaction parameters: {}", params),
        }
    }
}

impl std::error::Error for ToolError {}

impl From<anyhow::Error> for ToolError {
    fn from(err: anyhow::Error) -> Self {
        ToolError::McpError(err)
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        ToolError::SerializationError(err)
    }
}

// Validate Address Tool
#[derive(Deserialize)]
pub struct ValidateAddressArgs {
    pub address: String,
}

pub struct ValidateAddressTool {
    client: Arc<Mutex<FoundryMcpClient>>,
}

impl ValidateAddressTool {
    pub fn new(client: Arc<Mutex<FoundryMcpClient>>) -> Self {
        Self { client }
    }
}

impl Tool for ValidateAddressTool {
    const NAME: &'static str = "validate_address";
    type Error = ToolError;
    type Args = ValidateAddressArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "validate_address".to_string(),
            description: "Validate an Ethereum address format and checksum".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "The Ethereum address to validate"
                    }
                },
                "required": ["address"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let client = self.client.lock().await;
        let result = client.validate_address(&args.address).await?;
        Ok(result)
    }
}

// Send Transaction Tool
#[derive(Deserialize)]
pub struct SendTransactionArgs {
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: Option<String>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u128>,
}

pub struct SendTransactionTool {
    client: Arc<Mutex<FoundryMcpClient>>,
}

impl SendTransactionTool {
    pub fn new(client: Arc<Mutex<FoundryMcpClient>>) -> Self {
        Self { client }
    }
}

impl Tool for SendTransactionTool {
    const NAME: &'static str = "send_transaction";
    type Error = ToolError;
    type Args = SendTransactionArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "send_transaction".to_string(),
            description: "Send an Ethereum transaction with specified parameters".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "from": {
                        "type": "string",
                        "description": "Sender address"
                    },
                    "to": {
                        "type": "string",
                        "description": "Recipient address"
                    },
                    "value": {
                        "type": "string",
                        "description": "Amount of ETH to send (in wei)"
                    },
                    "data": {
                        "type": "string",
                        "description": "Transaction data (hex encoded)"
                    },
                    "gas_limit": {
                        "type": "number",
                        "description": "Gas limit for the transaction"
                    },
                    "gas_price": {
                        "type": "number",
                        "description": "Gas price (in wei)"
                    }
                },
                "required": ["from", "to", "value"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let client = self.client.lock().await;
        let result = client.send_transaction(
            &args.from,
            &args.to,
            &args.value,
            args.data.as_deref(),
            args.gas_limit,
            args.gas_price,
        ).await?;
        Ok(result)
    }
}

// Balance Tool
#[derive(Deserialize)]
pub struct BalanceArgs {
    pub address: String,
}

pub struct BalanceTool {
    client: Arc<Mutex<FoundryMcpClient>>,
}

impl BalanceTool {
    pub fn new(client: Arc<Mutex<FoundryMcpClient>>) -> Self {
        Self { client }
    }
}

impl Tool for BalanceTool {
    const NAME: &'static str = "balance";
    type Error = ToolError;
    type Args = BalanceArgs;
    type Output = serde_json::Value;    

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "balance".to_string(),
            description: "Check the balance of an Ethereum address".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "The Ethereum address to check balance for"
                    }
                },
                "required": ["address"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let client = self.client.lock().await;
        let result = client.balance(&args.address).await?;
        Ok(result)
    }
}

// Get Contract Code Tool

#[derive(Deserialize)]
pub struct GetContractCodeArgs {
    pub address: String,
}

pub struct GetContractCodeTool {
    client: Arc<Mutex<FoundryMcpClient>>,
}

impl GetContractCodeTool {
    pub fn new(client: Arc<Mutex<FoundryMcpClient>>) -> Self {
        Self { client }
    }
}

impl Tool for GetContractCodeTool {

    const NAME: &'static str = "get_contract_code";
    type Error = ToolError;
    type Args = GetContractCodeArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_contract_code".to_string(),
            description: "Get the contract code of an Ethereum address".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "The Ethereum address to get the contract code for"
                    }
                },
                "required": ["address"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {

        let client = self.client.lock().await;
        let result = client.get_contract_code(&args.address).await?;
        Ok(result)
    }
}

// Web Search Tool

#[derive(Deserialize)]
pub struct WebSearchArgs {
    pub query: String,
    pub count: Option<u32>,
    pub country: Option<String>,
    pub search_lang: Option<String>,
}

pub struct WebSearchTool {
    brave_search_api_key: String,
}

impl WebSearchTool {

    pub fn new(brave_search_api_key: String) -> Self {
        Self { brave_search_api_key }
    }

    async fn search(&self, query: &str, ) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();
        
        let response = client.get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", &self.brave_search_api_key)
            .query(&[("q", query)])
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}

impl Tool for WebSearchTool {

    const NAME: &'static str = "web_search";
    type Error = ToolError;
    type Args = WebSearchArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "web_search".to_string(),
            description: "Search the web for information using Brave Search API".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The query to search the web for"
                    },
                    "count": {
                        "type": "number",
                        "description": "Number of results to return (default: 20)"
                    },
                    "country": {
                        "type": "string",
                        "description": "Country code for localized results (e.g., 'us')"
                    },
                    "search_lang": {
                        "type": "string",
                        "description": "Search language (e.g., 'en')"
                    }
                },
                "required": ["query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let result = self.search(&args.query).await?;
        Ok(result)
    }
}


// Tool collection for managing all available tools
pub struct McpToolSet {
    pub validate_address: ValidateAddressTool,
    pub send_transaction: SendTransactionTool,
    pub balance: BalanceTool,
    pub web_search: WebSearchTool,
}

impl McpToolSet {
    pub fn new(client: Arc<Mutex<FoundryMcpClient>>, brave_search_api_key: String) -> Self {
        Self {
            validate_address: ValidateAddressTool::new(client.clone()),
            send_transaction: SendTransactionTool::new(client.clone()),
            balance: BalanceTool::new(client.clone()),
            web_search: WebSearchTool::new(brave_search_api_key),
        }
    }

    pub fn list_tools(&self) -> Vec<String> {
        vec![
            "validate_address".to_string(),
            "send_transaction".to_string(),
            "balance".to_string(),
            "web_search".to_string(),
        ]
    }

    pub async fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            self.validate_address.definition("".to_string()).await,
            self.send_transaction.definition("".to_string()).await,
            self.balance.definition("".to_string()).await,
            self.web_search.definition("".to_string()).await,
        ]
    }
}

// Helper function to create a tool set with a new MCP client
pub async fn create_mcp_tool_set(brave_search_api_key: String) -> Result<McpToolSet> {
    let client = FoundryMcpClient::new().await?;
    let client = Arc::new(Mutex::new(client));
    Ok(McpToolSet::new(client, brave_search_api_key))
}
