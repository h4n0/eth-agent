use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParam, ClientInfo, ServerNotification, ServerRequest},
    service::{NotificationContext, RoleClient, Service, ServiceExt},
    transport::TokioChildProcess,
};
use tokio::process::Command;
use tracing::{debug, info};
use std::future::Future;

// Simple service implementation for the client
#[derive(Debug, Clone)]
struct SimpleClientService;

impl Service<RoleClient> for SimpleClientService {
    fn handle_request(
        &self,
        _request: ServerRequest,
        _context: rmcp::service::RequestContext<RoleClient>,
    ) -> impl Future<Output = Result<rmcp::model::ClientResult, rmcp::ErrorData>> + Send + '_ {
        async move {
            // This is a client service, so we don't handle server requests
            Err(rmcp::ErrorData {
                code: rmcp::model::ErrorCode::METHOD_NOT_FOUND,
                message: "Client service does not handle server requests".into(),
                data: None,
            })
        }
    }

    fn handle_notification(
        &self,
        _notification: ServerNotification,
        _context: NotificationContext<RoleClient>,
    ) -> impl Future<Output = Result<(), rmcp::ErrorData>> + Send + '_ {
        async move {
            // This is a client service, so we don't handle server notifications
            Ok(())
        }
    }

    fn get_info(&self) -> ClientInfo {
        ClientInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: rmcp::model::ClientCapabilities::default(),
            client_info: rmcp::model::Implementation {
                name: "eth-agent".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }
}

pub struct FoundryMcpClient {
    service: rmcp::service::RunningService<RoleClient, SimpleClientService>,
}

impl FoundryMcpClient {
    pub async fn new() -> Result<Self> {
        info!("Starting foundry-mcp server as child process");
        
        // Use cargo run to start the foundry-mcp server as a child process
        let mut command = Command::new("cargo");
        command.args(["run", "--bin", "foundry-mcp"]);
        
        // Suppress server output by redirecting stderr to null
        // (stdout is used for MCP communication, so we keep that)
        command.stderr(std::process::Stdio::null());
        
        // Create the service using TokioChildProcess
        let transport = TokioChildProcess::new(command)
            .map_err(|e| anyhow::anyhow!("Failed to start foundry-mcp server: {}", e))?;
            
        let service = SimpleClientService.serve(transport).await
            .map_err(|e| anyhow::anyhow!("Failed to establish MCP connection: {}", e))?;

        debug!("Connected to server: {:#?}", service.peer().peer_info());

        let client = Self { service };
        
        Ok(client)
    }

    pub async fn balance(&self, address: &str) -> Result<serde_json::Value> {
        let tool_result = self.service.peer()
            .call_tool(CallToolRequestParam {
                name: "balance".into(),
                arguments: serde_json::json!({ "address": address }).as_object().cloned(),
            })
            .await?;
        
        debug!("Balance tool result: {tool_result:#?}");
        
        Ok(serde_json::to_value(tool_result)?)
    }

    pub async fn validate_address(&self, address: &str) -> Result<serde_json::Value> {
        let tool_result = self.service.peer()
            .call_tool(CallToolRequestParam {
                name: "validate_address".into(),
                arguments: serde_json::json!({ "address": address }).as_object().cloned(),
            })
            .await?;
        
        debug!("Validate address tool result: {tool_result:#?}");
        
        Ok(serde_json::to_value(tool_result)?)
    }

    pub async fn send_transaction(
        &self,
        from: &str,
        to: &str,
        value: &str,
        data: Option<&str>,
        gas_limit: Option<u64>,
        gas_price: Option<u128>,
    ) -> Result<serde_json::Value> {
        let mut arguments = serde_json::json!({
            "from": from,
            "to": to,
            "value": value,
        });

        if let Some(data) = data {
            arguments["data"] = serde_json::json!(data);
        }

        if let Some(gas_limit) = gas_limit {
            arguments["gas_limit"] = serde_json::json!(gas_limit);
        }

        if let Some(gas_price) = gas_price {
            arguments["gas_price"] = serde_json::json!(gas_price);
        }

        let tool_result = self.service.peer()
            .call_tool(CallToolRequestParam {
                name: "send_transaction".into(),
                arguments: arguments.as_object().cloned(),
            })
            .await?;
        
        debug!("Send transaction tool result: {tool_result:#?}");
        
        Ok(serde_json::to_value(tool_result.content)?)
    }

    pub async fn get_contract_code(&self, address: &str) -> Result<serde_json::Value> {
        let tool_result = self.service.peer()
            .call_tool(CallToolRequestParam {
                name: "get_contract_code".into(),
                arguments: serde_json::json!({ "address": address }).as_object().cloned(),
            })
            .await?;

        debug!("Get contract code tool result: {tool_result:#?}");
        
        Ok(serde_json::to_value(tool_result)?)
    }

    pub async fn erc20_balance(&self, address: &str, token_address: &str) -> Result<serde_json::Value> {
        let tool_result = self.service.peer()
            .call_tool(CallToolRequestParam {
                name: "erc20_balance".into(),
                arguments: serde_json::json!({ "address": address, "token_address": token_address }).as_object().cloned(),
            })
            .await?;

        debug!("ERC20 balance tool result: {tool_result:#?}");
        
        Ok(serde_json::to_value(tool_result)?)
    }

    #[allow(dead_code)]
    pub async fn list_tools(&self) -> Result<serde_json::Value> {
        let tools = self.service.peer().list_tools(Default::default()).await?;
        Ok(serde_json::to_value(tools)?)
    }

    #[allow(dead_code)]
    pub async fn cancel(self) -> Result<()> {
        self.service.cancel().await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn is_alive(&self) -> bool {
        // Check if the service is still connected by trying to list tools
        match self.service.peer().list_tools(Default::default()).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
} 