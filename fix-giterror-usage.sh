#!/bin/bash

# Fix the GitError usage in git_state_analyzer.rs
file="crates/gitai-analysis/src/architectural_impact/git_state_analyzer.rs"

echo "Fixing GitError usage in $file..."

# Replace GitError::CommandFailed(format!(...)) with just the format!(...) part
# since GitAIError::Git expects a String, not a GitError

# Fix pattern 1: GitAIError::Git(GitError::CommandFailed(format!(...)))
sed -i '' 's/GitAIError::Git(GitError::CommandFailed(format!(/GitAIError::Git(format!(/g' "$file"

# Fix pattern 2: GitAIError::Git(GitError::CommandFailed("...".to_string()))
sed -i '' 's/GitAIError::Git(GitError::CommandFailed(/GitAIError::Git(/g' "$file"

# Remove one extra closing parenthesis that's no longer needed
sed -i '' 's/))))/)))/g' "$file"

echo "Fixed GitError usage in git_state_analyzer.rs"
