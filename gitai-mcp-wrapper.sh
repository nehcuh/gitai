#!/bin/bash

# GitAI MCP Wrapper Script for Debugging
# This script wraps the gitai mcp command to provide better error handling

# Set up environment
export RUST_LOG="${RUST_LOG:-info}"
export GITAI_CONFIG_PATH="${GITAI_CONFIG_PATH:-$HOME/.config/gitai/config.toml}"

# Log startup
echo "🚀 Starting GitAI MCP wrapper..." >&2
echo "📁 Config: $GITAI_CONFIG_PATH" >&2
echo "📊 Log level: $RUST_LOG" >&2

# Check if config exists
if [ ! -f "$GITAI_CONFIG_PATH" ]; then
    echo "⚠️ Config file not found at $GITAI_CONFIG_PATH" >&2
    echo "🔧 Running gitai init to create config..." >&2
    /Users/huchen/Projects/gitai/target/release/gitai init >&2
fi

# Check if binary exists and is executable
GITAI_BIN="/Users/huchen/Projects/gitai/target/release/gitai"
if [ ! -x "$GITAI_BIN" ]; then
    echo "❌ GitAI binary not found or not executable at $GITAI_BIN" >&2
    exit 1
fi

# Run the MCP server
exec "$GITAI_BIN" mcp --transport stdio
