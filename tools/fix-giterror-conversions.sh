#!/bin/bash
# Fix GitError conversions in gitai-analysis crate

FILE="/Users/huchen/Projects/gitai/crates/gitai-analysis/src/architectural_impact/git_state_analyzer.rs"

echo "Fixing GitError conversions..."

# Replace GitAIError::Git(GitError::CommandFailed(...)) with GitAIError::Git(...)
sed -i '' 's/GitAIError::Git(GitError::CommandFailed(\(.*\)))/GitAIError::Git(\1)/g' "$FILE"

echo "GitError conversion fixes complete!"
