#!/bin/bash

# GitAI 导入路径更新脚本
# 将旧的src/导入路径更新为新的crates/路径

set -e

echo "========================================="
echo "GitAI 导入路径更新脚本"
echo "========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 统计变更
TOTAL_CHANGES=0

# 更新配置相关的导入
echo "1. 更新配置模块导入..."
FILES_CHANGED=$(rg -l "use crate::config" --type rust 2>/dev/null | wc -l | tr -d ' ')
if [ "$FILES_CHANGED" -gt 0 ]; then
    echo -e "${YELLOW}  找到 $FILES_CHANGED 个文件需要更新${NC}"
    rg -l "use crate::config" --type rust 2>/dev/null | while read -r file; do
        sed -i.bak 's/use crate::config/use gitai_core::config/g' "$file"
        echo "    ✓ 更新: $file"
    done
    TOTAL_CHANGES=$((TOTAL_CHANGES + FILES_CHANGED))
else
    echo -e "${GREEN}  ✓ 无需更新${NC}"
fi

# 更新domain接口导入
echo "2. 更新domain接口导入..."
FILES_CHANGED=$(rg -l "use crate::domain::interfaces" --type rust 2>/dev/null | wc -l | tr -d ' ')
if [ "$FILES_CHANGED" -gt 0 ]; then
    echo -e "${YELLOW}  找到 $FILES_CHANGED 个文件需要更新${NC}"
    rg -l "use crate::domain::interfaces" --type rust 2>/dev/null | while read -r file; do
        sed -i.bak 's/use crate::domain::interfaces/use gitai_core::interfaces/g' "$file"
        echo "    ✓ 更新: $file"
    done
    TOTAL_CHANGES=$((TOTAL_CHANGES + FILES_CHANGED))
else
    echo -e "${GREEN}  ✓ 无需更新${NC}"
fi

# 更新domain服务导入
echo "3. 更新domain服务导入..."
FILES_CHANGED=$(rg -l "use crate::domain::services" --type rust 2>/dev/null | wc -l | tr -d ' ')
if [ "$FILES_CHANGED" -gt 0 ]; then
    echo -e "${YELLOW}  找到 $FILES_CHANGED 个文件需要更新${NC}"
    rg -l "use crate::domain::services" --type rust 2>/dev/null | while read -r file; do
        sed -i.bak 's/use crate::domain::services/use gitai_core::services/g' "$file"
        echo "    ✓ 更新: $file"
    done
    TOTAL_CHANGES=$((TOTAL_CHANGES + FILES_CHANGED))
else
    echo -e "${GREEN}  ✓ 无需更新${NC}"
fi

# 更新git模块导入
echo "4. 更新git模块导入..."
FILES_CHANGED=$(rg -l "use crate::git::" --type rust 2>/dev/null | wc -l | tr -d ' ')
if [ "$FILES_CHANGED" -gt 0 ]; then
    echo -e "${YELLOW}  找到 $FILES_CHANGED 个文件需要更新${NC}"
    rg -l "use crate::git::" --type rust 2>/dev/null | while read -r file; do
        sed -i.bak 's/use crate::git::/use gitai_core::git::/g' "$file"
        echo "    ✓ 更新: $file"
    done
    TOTAL_CHANGES=$((TOTAL_CHANGES + FILES_CHANGED))
else
    echo -e "${GREEN}  ✓ 无需更新${NC}"
fi

# 更新错误类型导入
echo "5. 更新错误类型导入..."
FILES_CHANGED=$(rg -l "use crate::error::" --type rust 2>/dev/null | wc -l | tr -d ' ')
if [ "$FILES_CHANGED" -gt 0 ]; then
    echo -e "${YELLOW}  找到 $FILES_CHANGED 个文件需要更新${NC}"
    rg -l "use crate::error::" --type rust 2>/dev/null | while read -r file; do
        sed -i.bak 's/use crate::error::/use gitai_types::/g' "$file"
        echo "    ✓ 更新: $file"
    done
    TOTAL_CHANGES=$((TOTAL_CHANGES + FILES_CHANGED))
else
    echo -e "${GREEN}  ✓ 无需更新${NC}"
fi

# 清理备份文件
echo ""
echo "6. 清理备份文件..."
find . -name "*.bak" -type f -delete
echo -e "${GREEN}  ✓ 备份文件已清理${NC}"

# 总结
echo ""
echo "========================================="
echo "更新完成"
echo "========================================="
echo ""
echo "总计更新: $TOTAL_CHANGES 处导入"
echo ""
echo "建议下一步："
echo "1. 运行 'cargo check' 验证编译"
echo "2. 运行 'cargo test' 验证功能"
echo "3. 提交变更到版本控制"

exit 0
