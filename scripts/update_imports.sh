#!/bin/bash

# 批量更新src/下对config和git模块的引用

echo "批量更新导入路径..."

# 查找并替换use crate::config
find src -type f -name "*.rs" -exec sed -i '' 's/use crate::config::/use gitai_core::config::/g' {} \;
find src -type f -name "*.rs" -exec sed -i '' 's/use crate::config;/use gitai_core::config::Config;/g' {} \;

# 查找并替换use crate::git  
find src -type f -name "*.rs" -exec sed -i '' 's/use crate::git::/use gitai_core::git_impl::/g' {} \;
find src -type f -name "*.rs" -exec sed -i '' 's/use crate::git;/use gitai_core::git_impl;/g' {} \;

# 替换crate::config和crate::git
find src -type f -name "*.rs" -exec sed -i '' 's/crate::config::/gitai_core::config::/g' {} \;
find src -type f -name "*.rs" -exec sed -i '' 's/crate::git::/gitai_core::git_impl::/g' {} \;

echo "导入路径更新完成"
