# ETH Agent Makefile
# Coordinates running the foundry-mcp server and eth-agent client

.PHONY: help build run server client clean test

# Default target
help:
	@echo "ETH Agent Development Commands:"
	@echo "  make build    - Build both binaries"
	@echo "  make run      - Run both server and client together"
	@echo "  make server   - Run only the foundry-mcp server"
	@echo "  make client   - Run only the eth-agent client"
	@echo "  make test     - Run tests"
	@echo "  make clean    - Clean build artifacts"
	@echo ""
	@echo "Environment variables needed:"
	@echo "  ANTHROPIC_API_KEY      - Claude API key"
	@echo "  BRAVE_SEARCH_API_KEY   - Brave Search API key"

# Build both binaries
build:
	@echo "Building eth-agent and foundry-mcp..."
	cargo build

# Build release versions
build-release:
	@echo "Building release versions..."
	cargo build --release

# Run the eth-agent (which will automatically start foundry-mcp as child process)
run: build
	@echo "Starting eth-agent (which will spawn foundry-mcp server automatically)..."
	@echo "Press Ctrl+C to stop both processes"
	cargo run --bin eth-agent

# Run only the server (for development/debugging)
server:
	@echo "Starting foundry-mcp server..."
	cargo run --bin foundry-mcp

# Run only the client (assumes server is already running)
client:
	@echo "Starting eth-agent client..."
	cargo run --bin eth-agent

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Development target - build and run with logging
dev: build
	@echo "Starting in development mode with debug logging..."
	RUST_LOG=debug cargo run --bin eth-agent

# Check environment variables
check-env:
	@echo "Checking required environment variables..."
	@if [ -z "$$ANTHROPIC_API_KEY" ]; then \
		echo "ERROR: ANTHROPIC_API_KEY is not set"; \
		exit 1; \
	fi
	@if [ -z "$$BRAVE_SEARCH_API_KEY" ]; then \
		echo "ERROR: BRAVE_SEARCH_API_KEY is not set"; \
		exit 1; \
	fi
	@echo "Environment variables are properly configured"

# Run with environment check
run-safe: check-env run