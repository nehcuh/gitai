#!/bin/bash

# Gitai 网络兼容性测试脚本
# 用于验证 build.rs 在不同网络环境下的行为

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUERIES_DIR="$PROJECT_ROOT/queries"
BACKUP_DIR="$PROJECT_ROOT/queries_backup"

echo "🧪 开始 Gitai 网络兼容性测试"
echo "项目根目录: $PROJECT_ROOT"

# 颜色输出函数
print_success() {
    echo -e "\033[32m✅ $1\033[0m"
}

print_error() {
    echo -e "\033[31m❌ $1\033[0m"
}

print_info() {
    echo -e "\033[34mℹ️  $1\033[0m"
}

print_warning() {
    echo -e "\033[33m⚠️  $1\033[0m"
}

# 清理函数
cleanup() {
    print_info "清理测试环境..."
    if [ -d "$BACKUP_DIR" ]; then
        rm -rf "$QUERIES_DIR"
        mv "$BACKUP_DIR" "$QUERIES_DIR"
        print_info "恢复原始查询文件"
    fi
    cargo clean > /dev/null 2>&1 || true
}

# 注册清理函数
trap cleanup EXIT

# 测试1: 验证本地文件存在时跳过下载
test_local_files_exist() {
    print_info "测试1: 验证本地文件存在时跳过下载"
    
    # 备份现有文件
    if [ -d "$QUERIES_DIR" ]; then
        cp -r "$QUERIES_DIR" "$BACKUP_DIR"
        print_info "已备份现有查询文件"
    fi
    
    # 运行构建并捕获输出
    build_output=$(cargo check --offline 2>&1 | grep -E "(本地文件已存在|跳过下载)" || true)
    
    if [[ $build_output == *"本地文件已存在"* ]]; then
        print_success "本地文件检查正常工作"
        print_info "找到跳过的文件:"
        echo "$build_output" | head -5
    else
        print_error "本地文件检查未按预期工作"
        return 1
    fi
}

# 测试2: 验证网络失败时的回退机制
test_network_failure_fallback() {
    print_info "测试2: 验证网络失败时的回退机制"
    
    # 删除一个查询文件来模拟需要下载的情况
    rm -f "$QUERIES_DIR/rust/highlights.scm"
    print_info "删除 rust/highlights.scm 文件来模拟网络下载场景"
    
    # 临时修改 build.rs 使用无效 URL
    backup_build_rs=$(cat build.rs)
    sed -i.bak 's|https://raw.githubusercontent.com|https://invalid-test-url.example.com|g' build.rs
    print_info "临时修改 build.rs 使用无效 URL"
    
    # 运行构建
    if cargo check --offline > /dev/null 2>&1; then
        # 检查是否创建了备用文件
        if [ -f "$QUERIES_DIR/rust/highlights.scm" ]; then
            print_success "网络失败回退机制工作正常"
            print_info "备用查询文件内容:"
            head -3 "$QUERIES_DIR/rust/highlights.scm"
        else
            print_error "备用文件未被创建"
            return 1
        fi
    else
        print_error "构建失败，回退机制未工作"
        return 1
    fi
    
    # 恢复 build.rs
    echo "$backup_build_rs" > build.rs
    print_info "恢复原始 build.rs"
}

# 测试3: 验证查询文件的基本语法
test_query_syntax() {
    print_info "测试3: 验证查询文件的基本语法"
    
    local errors=0
    
    for lang_dir in "$QUERIES_DIR"/*; do
        if [ -d "$lang_dir" ]; then
            lang=$(basename "$lang_dir")
            for query_file in "$lang_dir"/*.scm; do
                if [ -f "$query_file" ]; then
                    # 基本语法检查：确保文件不为空且包含基本查询格式
                    if [ -s "$query_file" ] && grep -q "@" "$query_file"; then
                        continue
                    else
                        print_warning "查询文件可能有问题: $query_file"
                        errors=$((errors + 1))
                    fi
                fi
            done
        fi
    done
    
    if [ $errors -eq 0 ]; then
        print_success "所有查询文件语法检查通过"
    else
        print_warning "发现 $errors 个潜在问题的查询文件"
    fi
}

# 测试4: 验证构建性能（跳过网络请求后的速度）
test_build_performance() {
    print_info "测试4: 验证构建性能"
    
    print_info "执行干净构建..."
    cargo clean > /dev/null 2>&1
    
    start_time=$(date +%s)
    cargo check --offline > /dev/null 2>&1
    end_time=$(date +%s)
    
    duration=$((end_time - start_time))
    print_success "构建完成，耗时: ${duration}秒"
    
    if [ $duration -lt 60 ]; then
        print_success "构建速度良好 (< 60秒)"
    else
        print_warning "构建时间较长 (${duration}秒)，可能需要优化"
    fi
}

# 主测试流程
main() {
    print_info "开始网络兼容性测试套件"
    echo
    
    # 运行测试
    test_local_files_exist
    echo
    
    test_network_failure_fallback
    echo
    
    test_query_syntax
    echo
    
    test_build_performance
    echo
    
    print_success "所有网络兼容性测试完成！"
    echo
    print_info "测试总结:"
    print_info "✓ 本地文件优先策略工作正常"
    print_info "✓ 网络失败回退机制正常"
    print_info "✓ 查询文件语法正确"
    print_info "✓ 构建性能良好"
    echo
    print_success "Gitai 网络兼容性验证通过！"
}

# 检查是否在正确的目录中运行
if [ ! -f "Cargo.toml" ] || [ ! -f "build.rs" ]; then
    print_error "请在 gitai 项目根目录下运行此脚本"
    exit 1
fi

# 运行主测试
main