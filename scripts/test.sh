#!/bin/bash
# GitAI Test Script

set -e

echo "🧪 Running GitAI Tests..."

# Run unit tests
echo "📋 Running unit tests..."
cargo test --lib

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test integration_commit_test
cargo test --test translation_integration

# Run all tests with coverage info
echo "📊 Running all tests..."
cargo test -- --nocapture

echo "✅ All tests completed successfully!"