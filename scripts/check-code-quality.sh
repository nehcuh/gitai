#!/bin/bash

# GitAI 代码质量检查脚本
# 用于检测技术债务和代码质量问题

set -e

echo "========================================="
echo "GitAI 代码质量检查"
echo "========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 结果变量
ISSUES_FOUND=0

# 1. 检查 Box<dyn Error> 使用
echo "1. 检查 Box<dyn Error> 使用..."
BOX_DYN_COUNT=$(rg "Box<dyn std::error::Error" --type rust 2>/dev/null | wc -l | tr -d ' ')
if [ "$BOX_DYN_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}⚠ 发现 $BOX_DYN_COUNT 个 Box<dyn Error> 使用${NC}"
    echo "   详细分布："
    rg "Box<dyn std::error::Error" --type rust -c 2>/dev/null | sort -t: -k2 -rn | head -10 | sed 's/^/     /'
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    echo -e "${GREEN}✓ 未发现 Box<dyn Error> 使用${NC}"
fi
echo ""

# 2. 检查重复的 config.rs 文件
echo "2. 检查重复的 config.rs 文件..."
CONFIG_FILES=$(find . -type f -name "config.rs" -not -path "./target/*" 2>/dev/null | wc -l | tr -d ' ')
if [ "$CONFIG_FILES" -gt 1 ]; then
    echo -e "${YELLOW}⚠ 发现 $CONFIG_FILES 个 config.rs 文件${NC}"
    echo "   文件列表："
    find . -type f -name "config.rs" -not -path "./target/*" 2>/dev/null | sed 's/^/     /'
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    echo -e "${GREEN}✓ 只有 1 个 config.rs 文件${NC}"
fi
echo ""

# 3. 检查编译警告
echo "3. 检查编译警告..."
WARNINGS=$(cargo clippy --all-targets 2>&1 | grep -c "warning:" || true)
if [ "$WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}⚠ 发现 $WARNINGS 个编译警告${NC}"
    echo "   运行 'cargo clippy --all-targets' 查看详情"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    echo -e "${GREEN}✓ 无编译警告${NC}"
fi
echo ""

# 4. 检查未使用的依赖
echo "4. 检查未使用的依赖..."
if command -v cargo-udeps &> /dev/null; then
    UNUSED_DEPS=$(cargo +nightly udeps 2>&1 | grep -c "unused" || true)
    if [ "$UNUSED_DEPS" -gt 0 ]; then
        echo -e "${YELLOW}⚠ 发现未使用的依赖${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    else
        echo -e "${GREEN}✓ 无未使用的依赖${NC}"
    fi
else
    echo -e "${YELLOW}⚠ cargo-udeps 未安装，跳过检查${NC}"
    echo "   安装命令: cargo install cargo-udeps"
fi
echo ""

# 5. 检查测试覆盖率
echo "5. 运行测试..."
if cargo test --quiet 2>&1; then
    echo -e "${GREEN}✓ 所有测试通过${NC}"
else
    echo -e "${RED}✗ 测试失败${NC}"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
fi
echo ""

# 6. 检查代码格式
echo "6. 检查代码格式..."
if cargo fmt --all -- --check 2>&1; then
    echo -e "${GREEN}✓ 代码格式正确${NC}"
else
    echo -e "${YELLOW}⚠ 需要格式化代码${NC}"
    echo "   运行 'cargo fmt --all' 来格式化"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
fi
echo ""

# 7. 检查遗留的 src/ 目录代码
echo "7. 检查 src/ 目录中的遗留代码..."
SRC_RS_FILES=$(find src -name "*.rs" -type f 2>/dev/null | wc -l | tr -d ' ')
if [ "$SRC_RS_FILES" -gt 5 ]; then
    echo -e "${YELLOW}⚠ src/ 目录中有 $SRC_RS_FILES 个 Rust 文件${NC}"
    echo "   应考虑迁移到 crates/ 结构"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    echo -e "${GREEN}✓ src/ 目录代码量合理${NC}"
fi
echo ""

# 8. 统计实际完成度
echo "========================================="
echo "实际完成度评估"
echo "========================================="

# 计算完成度指标
TOTAL_CHECKS=7
PASSED_CHECKS=$((TOTAL_CHECKS - ISSUES_FOUND))
COMPLETION_PERCENT=$((PASSED_CHECKS * 100 / TOTAL_CHECKS))

echo ""
echo "检查项通过: $PASSED_CHECKS / $TOTAL_CHECKS"
echo "质量完成度: ${COMPLETION_PERCENT}%"
echo ""

# 基于错误处理的完成度调整
if [ "$BOX_DYN_COUNT" -gt 0 ]; then
    ADJUSTED_COMPLETION=$((35 + (100 - BOX_DYN_COUNT * 100 / 353) * 20 / 100))
else
    ADJUSTED_COMPLETION=55  # 如果Box<dyn Error>已清理，基础完成度提升到55%
fi

echo "调整后的项目完成度: ${ADJUSTED_COMPLETION}%"
echo "(基于353个Box<dyn Error>的清理进度)"
echo ""

# 总结
echo "========================================="
if [ "$ISSUES_FOUND" -eq 0 ]; then
    echo -e "${GREEN}✓ 所有质量检查通过！${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ 发现 $ISSUES_FOUND 个质量问题需要解决${NC}"
    echo ""
    echo "优先解决顺序："
    echo "1. Box<dyn Error> 迁移 (当前: $BOX_DYN_COUNT/353)"
    echo "2. 清理重复的 config.rs 文件 (当前: $CONFIG_FILES 个)"
    echo "3. 修复编译警告"
    echo "4. 完成架构迁移 (src/ -> crates/)"
    exit 1
fi
