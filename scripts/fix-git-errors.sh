#!/bin/bash

echo "修复GitAIError::Git的错误使用..."

# 查找所有包含GitAIError::Git(字符串)的文件并修复
find crates -name "*.rs" -type f | while read file; do
    # 备份原文件
    cp "$file" "$file.bak" 2>/dev/null
    
    # 替换GitAIError::Git(字符串)为GitAIError::Git(GitError::CommandFailed(字符串))
    sed -i '' 's/GitAIError::Git(\([^)]*\))/GitAIError::Git(GitError::CommandFailed(\1))/g' "$file"
    
    # 如果文件被修改了，检查是否需要导入GitError
    if ! diff -q "$file" "$file.bak" > /dev/null 2>&1; then
        # 检查是否已经导入了GitError
        if ! grep -q "use.*GitError" "$file"; then
            # 如果有use gitai_types，在其后添加GitError导入
            if grep -q "use gitai_types::{" "$file"; then
                sed -i '' 's/use gitai_types::{/use gitai_types::{error::GitError, /' "$file"
            elif grep -q "use gitai_types::" "$file"; then
                # 在第一个use gitai_types之后添加
                sed -i '' '/use gitai_types::/a\
use gitai_types::error::GitError;' "$file"
            fi
        fi
        echo "修复: $file"
        rm "$file.bak"
    else
        rm "$file.bak" 2>/dev/null
    fi
done

echo "修复完成！"
