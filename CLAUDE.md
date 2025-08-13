# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Prerequisites
- Rust 1.88+ (required for `rig` and `foundry` dependencies)
- Environment variables:
  ```bash
  export ANTHROPIC_API_KEY="sk-..."
  export BRAVE_SEARCH_API_KEY=""
  # Note: FOUNDRY_MCP_BINARY no longer needed with new Makefile approach
  ```

### Building
```bash
make build                     # Build both binaries (preferred)
cargo build                    # Alternative: direct cargo build
cargo build --bin eth-agent    # Build agent only
cargo build --bin foundry-mcp  # Build MCP server only
```

### Running
```bash
# Run the complete system (preferred)
make run                       # Start eth-agent (auto-spawns foundry-mcp server)

# Alternative methods
./start_eth_agent.sh          # Legacy bash script approach

# Run components separately for debugging
make server                    # Start MCP server only
make client                    # Start agent only (assumes server running)

# Development with debug logging
make dev                       # Run with RUST_LOG=debug

# Other useful commands
make help                      # Show all available commands
make clean                     # Clean build artifacts
make test                      # Run tests
make check-env                 # Verify environment variables
```

### Testing
```bash
# Working test cases in the REPL:
# - "What is the ETH balance of Alice"
# - "Send 10 ETH from Alice to Bob"  
# - "What is the USDT balance of Eve"
# - Web search queries
```

## Architecture Overview

This is a ReAct-based Ethereum agent system with a multi-process architecture:

### Core Components

**ETH Agent (`agent/`)**
- Main orchestrator using a Plan → Execute → Evaluate loop
- Uses Claude 3.5 Haiku for all AI inference (planning, execution, evaluation)
- Manages specialized sub-agents for different domains
- Implements retry logic with configurable evaluation thresholds (default: 70/100)

**MCP Server (`foundry-mcp/`)**
- Separate process providing blockchain tools via Model Context Protocol
- Communicates with agent via JSON-RPC over stdio
- Implements Ethereum operations using Alloy and Foundry

**Sub-Agents**
- `ethereum_agent`: Blockchain operations (balance, transactions, validation, ERC20)
- `search_agent`: Web search via Brave API

### Process Architecture
```
┌─────────────┐    stdio     ┌──────────────┐    RPC      ┌─────────────┐
│  eth-agent  │ ←─────────→  │ foundry-mcp  │ ──────────→ │  Ethereum   │
│ (parent)    │   JSON-RPC   │   (child)    │   (Alloy)   │   Network   │
└─────────────┘              └──────────────┘             └─────────────┘

# Process management:
make run → cargo run --bin eth-agent → spawns foundry-mcp as child process
```

### Key Files
- `agent/main.rs`: CLI REPL interface and main entry point
- `agent/agent.rs`: Core ReAct loop implementation with planning, execution, evaluation
- `agent/mcp_client.rs`: MCP protocol client for blockchain tool communication
- `agent/tools.rs`: Tool definitions for Rig framework integration
- `foundry-mcp/foundry_service.rs`: MCP server with blockchain tool implementations

## Development Patterns

### Adding New Blockchain Tools
1. Add tool function to `foundry-mcp/foundry_service.rs` with `#[tool]` attribute
2. Create corresponding method in `agent/mcp_client.rs`
3. Add tool wrapper in `agent/tools.rs` for Rig integration
4. Update agent preamble in `agent/agent.rs` to document new tool

### Known Addresses for Testing
- Alice: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- Bob: `0x70997970C51812dc3A010C7d01b50e0d17dc79C8`
- Eve: `0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045`

### Known ERC20 Tokens
- USDC: `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
- USDT: `0xdAC17F958D2ee523a2206206994597C13D831ec7`
- DAI: `0x6B175474E89094C44Da98b954EedeAC495271d0F`

## Configuration Constants

- `ANTHROPIC_MODEL`: `"claude-3-5-haiku-20241022"`
- `EVALUATION_THRESHOLD`: `70` (score out of 100)
- `MAX_PLAN_RETRIES`: `3`

## Current Limitations

- Uniswap integration not implemented
- No universal contract call support
- Error handling needs improvement
- No structured output guarantees (uses JSON parsing hacks)
- Short-term memory only (no RAG)

## Workspace Structure

This is a Cargo workspace with two main binaries:
- `eth-agent` (in `agent/` directory)
- `foundry-mcp` (in `foundry-mcp/` directory)

The root `Cargo.toml` defines workspace members and shared dependencies.