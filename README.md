# ETH Agent with Foundry MCP Integration

A modular Ethereum agent system that uses the Model Context Protocol (MCP) to integrate with Foundry for blockchain interactions.

## Features

- **Multi-Step Agent Loop**: Advanced agent with planning, execution, and evaluation phases
- **Regex-based Prompt Parsing**: Intelligent parsing of natural language prompts using regex patterns
- **Max Steps Protection**: Configurable step limits to prevent infinite loops
- **Result Evaluation**: Automatic evaluation of execution results against original prompts
- **MCP-based Foundry Integration**: Uses the Model Context Protocol to communicate with Foundry tools
- **Address Validation**: Always validates Ethereum addresses before proceeding with operations
- **Balance Checking**: Check balances for native ETH and ERC20 tokens
- **Transaction Composition**: Compose, sign, and send transactions
- **Modular Architecture**: Clean separation between agent logic and blockchain interactions

## Architecture

The system consists of two main components:

1. **Foundry MCP Server** (`foundry-mcp/`): A standalone MCP server that provides Foundry tools
2. **ETH Agent** (`agent/`): A modular standalone binary with separate components for types, MCP client, action execution, and agent logic

### Multi-Step Agent Architecture

The ETH Agent implements a sophisticated multi-step execution loop:

1. **Planning Phase**: Uses regex patterns to parse natural language prompts into structured actions
2. **Execution Phase**: Executes planned steps with configurable max steps limit
3. **Evaluation Phase**: Evaluates results against original prompts for alignment

#### Supported Prompt Patterns

The agent recognizes the following patterns:
- `Transfer <amount> ETH to <address>` - Transfer ETH to a specific address
- `Deploy contract named <name>` - Deploy a contract with a given name
- `Call contract <address> function <name>` - Call a specific function on a contract
- `Read from contract <address> function <name>` - Read data from a contract
- `Batch call to contract <address>` - Execute batch operations on a contract

## Required Foundry Tools

The foundry-mcp implementation provides three essential tools:

1. **validate_address**: Validates Ethereum address format and returns checksum address
2. **check_balance**: Checks balance of a given address for native ETH or ERC20 tokens
3. **compose_transaction**: Composes, signs, and sends transactions

## Setup

### Prerequisites

- Rust 1.70+
- Foundry (for local development)
- An Ethereum RPC endpoint

### Environment Variables

Set the following environment variables:

```bash
# Required
export FOUNDRY_PRIVATE_KEY="0x..."  # Your private key (with 0x prefix)

# Optional (with defaults)
export FOUNDRY_RPC_URL="http://localhost:8545"  # Default: localhost:8545
export FOUNDRY_CHAIN_ID="1"                     # Default: 1 (mainnet)
export FOUNDRY_GAS_LIMIT="21000"                # Default: 21000
export FOUNDRY_GAS_PRICE="20000000000"          # Default: 20 gwei

# For the main agent
export OPENAI_API_KEY="sk-..."  # Required for the agent
```

### Building

```bash
cargo build
```

## Usage

### Running the Foundry MCP Server

```bash
# Run the standalone MCP server
cargo run --bin foundry-mcp

# Or run the example
cargo run --example mcp_usage
```

### Running the ETH Agent

The ETH Agent now includes an interactive CLI REPL (Read-Eval-Print Loop) that allows you to input commands directly.

```bash
# Run the main agent with interactive CLI
cargo run
```

### Running the Multi-Step Agent Example

To see the new multi-step agent features in action:

```bash
# Run the multi-step agent example
cargo run --example multi_step_agent
```

This example demonstrates:
- Regex-based prompt parsing
- Multi-step execution planning
- Max steps limit enforcement
- Result evaluation and alignment scoring
- Fallback parsing for unknown prompts

This will start an interactive session where you can type commands like:

```
🤖 ETH Agent CLI REPL
Type 'help' for available commands, 'quit' to exit
Network: mainnet
User address: 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6

eth-agent> help

📚 Available Commands:
  help, h          - Show this help message
  quit, exit, q    - Exit the REPL
  clear, cls       - Clear the screen

💡 Example Commands:
  Transfer 0.1 ETH to 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6
  Deploy a simple contract
  Call function 'mint' on contract 0x123...
  Read balance of 0x456...

🔧 Supported Actions:
  • Transfer ETH
  • Deploy contracts
  • Call contract functions
  • Read contract state
  • Batch operations

eth-agent> Transfer 0.1 ETH to 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6
✅ Command executed successfully!
📝 Transaction hash: 0x1234567890abcdef...
⛽ Gas used: 21000
🎯 Success: true

eth-agent> quit
Goodbye! 👋
```

#### Available Prompts

- **help, h**: Show help message with available prompts and examples
- **quit, exit, q**: Exit the REPL
- **clear, cls**: Clear the screen
- **Natural language prompts**: Type any request in natural language, such as:
  - "Transfer 0.1 ETH to 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
  - "Deploy a simple contract"
  - "Call function 'mint' on contract 0x123..."
  - "Read balance of 0x456..."

### Using the MCP Client Directly

```rust
use eth_agent_agent::FoundryMcpClient;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = FoundryMcpClient::new().await?;
    
    // Validate an address
    let validation = client.validate_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6").await?;
    println!("Address valid: {}", validation["is_valid"]);
    
    // Check balance
    let balance = client.check_balance("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6", None).await?;
    println!("Balance: {}", balance["balance"]);
    
    // Send a transaction
    let tx = client.compose_transaction(
        "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
        Some("1000000000000000000"), // 1 ETH in wei
        None,
        None,
        None,
    ).await?;
    println!("Transaction hash: {}", tx["transaction_hash"]);
    
    Ok(())
}
```

## MCP Protocol Integration

The system uses the Model Context Protocol (MCP) for communication between the agent and Foundry tools. This provides:

- **Standardized Interface**: MCP provides a standard way to define and call tools
- **Separation of Concerns**: The agent doesn't need to know about Foundry internals
- **Extensibility**: Easy to add new tools or modify existing ones
- **Interoperability**: Can be used with any MCP-compatible client

### Tool Definitions

The foundry-mcp server exposes these tools:

```json
{
  "name": "validate_address",
  "description": "Validate an Ethereum address format and return checksum address",
  "input_schema": {
    "type": "object",
    "properties": {
      "address": {
        "type": "string",
        "description": "The Ethereum address to validate"
      }
    },
    "required": ["address"]
  }
}
```

## Development

### Project Structure

```
eth-agent/
├── agent/                     # Main agent binary (modular structure)
│   ├── main.rs                # Main application entry point
│   ├── types.rs               # Data structures and types
│   ├── mcp_client.rs          # MCP client for foundry-mcp communication
│   ├── action_executor.rs     # Action execution logic
│   ├── agent.rs               # Main agent logic and command parsing
│   └── Cargo.toml             # Agent project configuration
├── foundry-mcp/               # Foundry MCP server (separate binary)
│   ├── foundry_service.rs     # MCP server implementation
│   ├── main.rs                # Standalone server binary
│   └── Cargo.toml             # Foundry MCP project configuration
├── examples/
│   └── mcp_usage.rs           # Example usage
└── README.md
```

### Building and Running

**Development:**
```bash
# Build both binaries
cargo build

# Run the main application (it will spawn foundry-mcp as subprocess)
cargo run --bin eth-agent

# Or run the foundry-mcp server directly
cargo run --bin foundry-mcp
```

**Production:**
```bash
# Build release binaries
cargo build --release

# The binaries will be available at:
# - target/release/eth-agent
# - target/release/foundry-mcp

# Set the binary path for foundry-mcp (optional)
export FOUNDRY_MCP_BINARY=/path/to/foundry-mcp

# Run the main application
./target/release/eth-agent
```

### Adding New Tools

To add a new Foundry tool:

1. Define the tool in `foundry-mcp/foundry_service.rs`
2. Implement the functionality in `foundry-mcp/foundry_service.rs`
3. Update the MCP client in `agent/main.rs`

## Security Considerations

- **Private Key Management**: Always use environment variables for private keys
- **Address Validation**: All addresses are validated before use
- **Gas Limits**: Set appropriate gas limits to prevent excessive spending
- **Network Selection**: Ensure you're using the correct network (mainnet/testnet)

## Things to Improve
- Use `rmcp` for MCP client
- Add config file to avoid rebuild in every config change

## License

MIT License 