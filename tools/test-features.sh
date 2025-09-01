#!/bin/bash

# GitAI 功能门控测试脚本
# 测试不同功能组合的编译和基本功能

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试结果统计
PASSED=0
FAILED=0
SKIPPED=0

# 打印函数
print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED++))
}

print_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED++))
}

print_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((SKIPPED++))
}

print_summary() {
    echo ""
    echo "========================================="
    echo -e "${GREEN}Passed: $PASSED${NC}"
    echo -e "${RED}Failed: $FAILED${NC}"
    echo -e "${YELLOW}Skipped: $SKIPPED${NC}"
    echo "========================================="
    
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}✅ All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed!${NC}"
        exit 1
    fi
}

# 测试编译函数
test_build() {
    local NAME=$1
    local FEATURES=$2
    
    print_test "Building $NAME variant..."
    
    if [ "$FEATURES" = "default" ]; then
        CMD="cargo build --release"
    elif [ "$FEATURES" = "none" ]; then
        CMD="cargo build --release --no-default-features"
    else
        CMD="cargo build --release --no-default-features --features $FEATURES"
    fi
    
    if $CMD > /dev/null 2>&1; then
        print_pass "Build $NAME succeeded"
        return 0
    else
        print_fail "Build $NAME failed"
        return 1
    fi
}

# 测试运行函数
test_run() {
    local NAME=$1
    local CMD=$2
    local EXPECTED_EXIT=$3
    
    print_test "Running: $CMD"
    
    set +e
    OUTPUT=$($CMD 2>&1)
    EXIT_CODE=$?
    set -e
    
    if [ "$EXPECTED_EXIT" = "any" ]; then
        print_pass "Command executed (exit code: $EXIT_CODE)"
        return 0
    elif [ $EXIT_CODE -eq $EXPECTED_EXIT ]; then
        print_pass "Command exited with expected code: $EXIT_CODE"
        return 0
    else
        print_fail "Expected exit code $EXPECTED_EXIT, got $EXIT_CODE"
        echo "Output: $OUTPUT"
        return 1
    fi
}

# 测试功能可用性
test_feature() {
    local FEATURE=$1
    local CMD=$2
    local SHOULD_WORK=$3
    
    print_test "Testing feature: $FEATURE"
    
    set +e
    OUTPUT=$($CMD 2>&1)
    EXIT_CODE=$?
    set -e
    
    if [ "$SHOULD_WORK" = "true" ]; then
        if echo "$OUTPUT" | grep -q "未启用\|not enabled\|not available"; then
            print_fail "$FEATURE should be available but isn't"
            return 1
        else
            print_pass "$FEATURE is available as expected"
            return 0
        fi
    else
        if echo "$OUTPUT" | grep -q "未启用\|not enabled\|not available"; then
            print_pass "$FEATURE is correctly unavailable"
            return 0
        else
            print_skip "$FEATURE might be available (unclear from output)"
            return 0
        fi
    fi
}

# 主测试流程
main() {
    echo "========================================="
    echo "GitAI Feature Flags Test Suite"
    echo "========================================="
    echo ""
    
    # 测试最小构建
    echo "### Testing Minimal Build ###"
    if test_build "minimal" "minimal"; then
        test_run "minimal help" "./target/release/gitai --help" 0
        test_run "minimal review" "./target/release/gitai review --help" 0
        test_run "minimal commit" "./target/release/gitai commit --help" 0
    fi
    echo ""
    
    # 测试仅 Rust 支持
    echo "### Testing Rust-only Build ###"
    if test_build "rust-only" "tree-sitter-rust"; then
        test_run "rust-only help" "./target/release/gitai --help" 0
        # 创建测试文件
        echo 'fn main() { println!("test"); }' > /tmp/test.rs
        test_feature "Rust parsing" "./target/release/gitai review --language rust" "true"
        rm -f /tmp/test.rs
    fi
    echo ""
    
    # 测试仅 Python 支持
    echo "### Testing Python-only Build ###"
    if test_build "python-only" "tree-sitter-python"; then
        test_run "python-only help" "./target/release/gitai --help" 0
        # 创建测试文件
        echo 'def test(): pass' > /tmp/test.py
        test_feature "Python parsing" "./target/release/gitai review --language python" "true"
        rm -f /tmp/test.py
    fi
    echo ""
    
    # 测试 AI 功能
    echo "### Testing AI Features ###"
    if test_build "ai" "ai,tree-sitter-rust"; then
        test_run "ai help" "./target/release/gitai --help" 0
        test_feature "AI features" "./target/release/gitai --ai status" "true"
    fi
    echo ""
    
    # 测试安全功能
    echo "### Testing Security Features ###"
    if test_build "security" "security"; then
        test_run "security help" "./target/release/gitai --help" 0
        test_feature "Security scan" "./target/release/gitai scan --help" "true"
    fi
    echo ""
    
    # 测试 MCP 功能
    echo "### Testing MCP Features ###"
    if test_build "mcp" "mcp"; then
        test_run "mcp help" "./target/release/gitai --help" 0
        test_feature "MCP server" "./target/release/gitai mcp --help" "true"
    fi
    echo ""
    
    # 测试完整功能集
    echo "### Testing Full Build ###"
    if test_build "full" "full"; then
        test_run "full help" "./target/release/gitai --help" 0
        test_feature "All features" "./target/release/gitai --help" "true"
    fi
    echo ""
    
    # 测试不兼容的功能组合
    echo "### Testing Feature Combinations ###"
    
    # Web 开发组合
    if test_build "web-dev" "tree-sitter-javascript,tree-sitter-typescript"; then
        print_pass "Web development features build successfully"
    fi
    
    # DevOps 组合
    if test_build "devops" "devops,metrics"; then
        print_pass "DevOps features build successfully"
    fi
    echo ""
    
    # 显示测试总结
    print_summary
}

# 运行主测试
main "$@"
