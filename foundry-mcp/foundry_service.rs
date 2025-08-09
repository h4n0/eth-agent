use alloy::{serde::WithOtherFields};
use alloy_provider::{network::AnyNetwork, Provider, RootProvider};
use rmcp::{
    schemars, tool, tool_router, tool_handler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ServerCapabilities, ServerInfo},
    ServerHandler,
};
use alloy_primitives::{Address, U256};
use alloy_rpc_types::eth::TransactionRequest;
use std::str::FromStr;
use hex;
use std::future::Future;
use serde_json::json;
use foundry_cli::{opts::RpcOpts, utils::LoadConfig};

#[derive(Clone)]
pub struct FoundryService {
    foundry_provider: RootProvider<AnyNetwork>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct BalanceRequest {
    #[schemars(description = "The address to check balance for")]
    pub address: String,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct ValidateAddressRequest {
    #[schemars(description = "The Ethereum address to validate")]
    pub address: String,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct SendTransactionRequest {
    #[schemars(description = "Sender address")]
    pub from: String,
    #[schemars(description = "Recipient address")]
    pub to: String,
    #[schemars(description = "Amount in wei")]
    pub value: String,
    #[schemars(description = "Transaction data (hex encoded)")]
    pub data: Option<String>,
    #[schemars(description = "Gas limit for the transaction")]
    pub gas_limit: Option<u64>,
    #[schemars(description = "Gas price (in wei)")]
    pub gas_price: Option<u128>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct GetContractCodeRequest {
    #[schemars(description = "The address to get the contract code for")]
    pub address: String,
}

#[tool_router]
impl FoundryService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = RpcOpts::default().load_config().unwrap();

        let provider = foundry_cli::utils::get_provider(&config).unwrap();
        
        Ok(Self {
            foundry_provider: provider,
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "Get the balance of an account in wei")]
    pub async fn balance(
        &self,
        Parameters(request): Parameters<BalanceRequest>,
    ) -> String {


        match Address::from_str(&request.address) {
            Ok(address) => {
                // FIXME error handling
                let balance = self.foundry_provider.get_balance(address).await.unwrap();

                let result = json!({
                    "success": true,
                    "address": address.to_string(),
                    "balance": balance.to_string(),
                    "unit": "wei",
                    "message": format!("Balance: {} wei", balance.to_string())
                });
                serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string())
            }
            Err(e) => {
                let result = json!({
                    "success": false,
                    "error": format!("Invalid address: {}", e),
                    "address": request.address
                });
                serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string())
            }
        }
    }

    #[tool(description = "Validate an Ethereum address and return checksum format")]
    pub async fn validate_address(
        &self,
        Parameters(request): Parameters<ValidateAddressRequest>,
    ) -> String {
        match Address::from_str(&request.address) {
            Ok(addr) => {
                let checksum = addr.to_string();
                let result = json!({
                    "success": true,
                    "valid": true,
                    "address": checksum,
                    "message": "Valid Ethereum address"
                });
                serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string())
            }
            Err(e) => {
                let result = json!({
                    "success": false,
                    "valid": false,
                    "error": format!("Invalid address: {}", e),
                    "address": request.address
                });
                serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string())
            }
        }
    }

    #[tool(description = "Send a transaction to an address")]
    pub async fn send_transaction(
        &self,
        Parameters(request): Parameters<SendTransactionRequest>,
    ) -> String {
        // Validate sender address
        let from_address = match Address::from_str(&request.from) {
            Ok(addr) => addr,
            Err(e) => {
                let result = json!({
                    "success": false,
                    "error": format!("Invalid sender address: {}", e),
                    "from": request.from
                });
                return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
            }
        };

        // Validate recipient address
        let to_address = match Address::from_str(&request.to) {
            Ok(addr) => addr,
            Err(e) => {
                let result = json!({
                    "success": false,
                    "error": format!("Invalid address: {}", e),
                    "to": request.to
                });
                return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
            }
        };
        

        let amount = match U256::from_str(&request.value) {
            Ok(amount) => amount,
            Err(e) => {
                let result = json!({
                    "success": false,
                    "error": format!("Invalid amount: {}", e),
                    "value": request.value
                });
                return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
            }
        };

        // Parse data if provided
        let data = if let Some(data_str) = &request.data {
            match hex::decode(data_str.trim_start_matches("0x")) {
                Ok(data) => data,
                Err(e) => {
                    let result = json!({
                        "success": false,
                        "error": format!("Invalid data format: {}", e),
                        "data": data_str
                    });
                    return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
                }
            }
        } else {
            vec![]
        };

        // Get the current nonce for the sender address
        let nonce = match self.foundry_provider.get_transaction_count(from_address).await {
            Ok(nonce) => nonce,
            Err(e) => {
                let result = json!({
                    "success": false,
                    "error": format!("Failed to get nonce: {}", e),
                    "from": request.from
                });
                return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
            }
        };

        // Create transaction request
        let mut tx_request = TransactionRequest::default()
            .to(to_address)
            .value(amount)
            .nonce(nonce)
            .from(from_address);

        if !data.is_empty() {
            tx_request = tx_request.input(data.into());
        }

        if let Some(gas_limit) = request.gas_limit {
            tx_request = tx_request.gas_limit(gas_limit);
        }

        if let Some(gas_price) = request.gas_price {
            tx_request = tx_request.gas_price(gas_price);
        }


        // Log the transaction details for debugging
        tracing::debug!("Sending transaction: from={}, to={}, value={}, nonce={}", 
                       request.from, request.to, request.value, nonce);

        // Send the transaction
        let tx_request = WithOtherFields::new(tx_request);
        let tx_response = self.foundry_provider.send_transaction(tx_request).await.map_err(|e| {
            let result = json!({
                "success": false,
                "error": format!("Failed to send transaction: {}", e),
                "from": request.from,
                "to": request.to
            });
            return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
        }).unwrap();
        
        tracing::debug!("Transaction sent with hash: {}", tx_response.tx_hash());
        
        let result = json!({
            "success": true,
            "transaction_hash": tx_response.tx_hash(),
            "from": request.from,
            "to": request.to,
            "value": request.value,
            "nonce": nonce,
            "message": "Transaction sent successfully"
        });
        
        return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
    }

    #[tool(description = "Check the contract code at an address")]
    pub async fn get_contract_code(
        &self,
        Parameters(request): Parameters<GetContractCodeRequest>,
    ) -> String {
        let code = self.foundry_provider.get_code_at(Address::from_str(&request.address).unwrap()).await.unwrap();
        if code.is_empty() {
            let result = json!({
                "success": false,
                "error": "No contract code found at address",
                "address": request.address
            });
            return serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string());
        } else {
            let result = json!({
                "success": true,
                "address": request.address,
                "code": code
            });
            serde_json::to_string(&result).unwrap_or_else(|_| "Error serializing response".to_string())
        }
    }
}

#[tool_handler]
impl ServerHandler for FoundryService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Foundry MCP server for Ethereum blockchain interactions".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
} 