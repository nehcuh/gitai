#!/bin/bash
# GitAI Build Script

set -e

echo "🔨 Building GitAI..."

# Format code
echo "📝 Formatting code..."
cargo fmt

# Check for linting issues
echo "🔍 Running clippy..."
cargo clippy -- -D warnings

# Build in debug mode
echo "🏗️ Building debug version..."
cargo build

# Run tests
echo "🧪 Running tests..."
cargo test

# Build in release mode
echo "🚀 Building release version..."
cargo build --release

echo "✅ Build completed successfully!"
echo "📍 Release binary location: target/release/gitai"