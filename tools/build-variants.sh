#!/bin/bash

# GitAI 多变体构建脚本
# 用于构建不同功能集的GitAI版本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 构建目录
BUILD_DIR="target/release"
DIST_DIR="dist"

# 创建分发目录
mkdir -p "$DIST_DIR"

# 打印带颜色的消息
print_msg() {
    echo -e "${GREEN}$1${NC}"
}

print_warn() {
    echo -e "${YELLOW}$1${NC}"
}

print_error() {
    echo -e "${RED}$1${NC}"
}

# 构建函数
build_variant() {
    local NAME=$1
    local FEATURES=$2
    local DESCRIPTION=$3
    
    echo ""
    print_msg "🔨 构建 $NAME 版本..."
    print_msg "   功能: $FEATURES"
    print_msg "   描述: $DESCRIPTION"
    
    if [ "$FEATURES" = "default" ]; then
        cargo build --release
    elif [ "$FEATURES" = "none" ]; then
        cargo build --release --no-default-features
    else
        cargo build --release --no-default-features --features "$FEATURES"
    fi
    
    # 移动到分发目录
    if [ -f "$BUILD_DIR/gitai" ]; then
        mv "$BUILD_DIR/gitai" "$DIST_DIR/gitai-$NAME"
        # 获取文件大小
        SIZE=$(ls -lh "$DIST_DIR/gitai-$NAME" | awk '{print $5}')
        print_msg "   ✅ 构建成功: gitai-$NAME ($SIZE)"
        # 保存功能列表到文件，便于查看
        ("$DIST_DIR/gitai-$NAME" features --format table > "$DIST_DIR/gitai-$NAME.features.txt") || true
    else
        print_error "   ❌ 构建失败"
        return 1
    fi
}

# 清理旧的构建
print_msg "🧹 清理旧的构建..."
cargo clean
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 构建各种变体
print_msg "🚀 开始构建 GitAI 变体..."

# 最小版本（无任何功能）
build_variant "minimal" "minimal" "最小功能集，仅核心功能"

# 仅 Rust 支持
build_variant "rust-only" "tree-sitter-rust" "仅 Rust 语言支持"

# 仅 Python 支持
build_variant "python-only" "tree-sitter-python" "仅 Python 语言支持"

# 仅 JavaScript 支持
build_variant "js-only" "tree-sitter-javascript" "仅 JavaScript 语言支持"

# Web 开发版（JS + TS）
build_variant "web" "tree-sitter-javascript,tree-sitter-typescript" "Web 开发版本（JavaScript + TypeScript）"

# 默认版本
build_variant "default" "default" "默认功能集（Rust + Python + JavaScript + AI）"

# AI 增强版（默认 + AI 功能）
build_variant "ai" "ai,tree-sitter-rust,tree-sitter-python,tree-sitter-javascript" "AI 增强版本"

# 安全扫描版
build_variant "security" "security,tree-sitter-rust,tree-sitter-python,tree-sitter-javascript" "安全扫描版本"

# DevOps 版
build_variant "devops" "devops,metrics,tree-sitter-rust,tree-sitter-python,tree-sitter-javascript" "DevOps 版本（含度量功能）"

# 完整版本
build_variant "full" "full" "完整功能集（所有功能）"

# MCP 服务器版
if cargo build --release --no-default-features --features mcp --bin gitai-mcp 2>/dev/null; then
    if [ -f "$BUILD_DIR/gitai-mcp" ]; then
        mv "$BUILD_DIR/gitai-mcp" "$DIST_DIR/gitai-mcp"
        SIZE=$(ls -lh "$DIST_DIR/gitai-mcp" | awk '{print $5}')
        print_msg "   ✅ MCP 服务器构建成功: gitai-mcp ($SIZE)"
    fi
fi

# 生成版本汇总
echo ""
print_msg "📊 构建汇总:"
echo ""
echo "版本名称          | 文件大小 | 说明"
echo "------------------|----------|----------------------------------------"
for file in "$DIST_DIR"/gitai-*; do
    if [ -f "$file" ]; then
        BASENAME=$(basename "$file")
        NAME=${BASENAME#gitai-}
        SIZE=$(ls -lh "$file" | awk '{print $5}')
        
        case "$NAME" in
            minimal)
                DESC="最小功能集"
                ;;
            rust-only)
                DESC="仅 Rust 语言支持"
                ;;
            python-only)
                DESC="仅 Python 语言支持"
                ;;
            js-only)
                DESC="仅 JavaScript 语言支持"
                ;;
            web)
                DESC="Web 开发（JS + TS）"
                ;;
            default)
                DESC="默认功能集"
                ;;
            ai)
                DESC="AI 增强版"
                ;;
            security)
                DESC="安全扫描版"
                ;;
            devops)
                DESC="DevOps 版本"
                ;;
            full)
                DESC="完整功能集"
                ;;
            mcp)
                DESC="MCP 服务器"
                ;;
            *)
                DESC="未知版本"
                ;;
        esac
        
        printf "%-17s | %8s | %s\n" "$NAME" "$SIZE" "$DESC"
    fi
done

echo ""
print_msg "✅ 所有构建完成！"
print_msg "📁 构建输出目录: $DIST_DIR"

# 创建安装脚本
cat > "$DIST_DIR/install.sh" << 'EOF'
#!/bin/bash

# GitAI 安装脚本

set -e

# 检测系统类型
OS=$(uname -s)
ARCH=$(uname -m)

echo "系统: $OS ($ARCH)"

# 选择版本
echo ""
echo "请选择要安装的版本:"
echo "1) minimal    - 最小功能集"
echo "2) default    - 默认功能集（推荐）"
echo "3) full       - 完整功能集"
echo "4) ai         - AI 增强版"
echo "5) security   - 安全扫描版"
echo "6) devops     - DevOps 版本"
echo "7) web        - Web 开发版"
echo "8) rust-only  - 仅 Rust 支持"
echo "9) python-only- 仅 Python 支持"

read -p "选择 (1-9) [2]: " choice
choice=${choice:-2}

case $choice in
    1) VARIANT="minimal";;
    2) VARIANT="default";;
    3) VARIANT="full";;
    4) VARIANT="ai";;
    5) VARIANT="security";;
    6) VARIANT="devops";;
    7) VARIANT="web";;
    8) VARIANT="rust-only";;
    9) VARIANT="python-only";;
    *) VARIANT="default";;
esac

BINARY="gitai-$VARIANT"

if [ ! -f "$BINARY" ]; then
    echo "错误: 找不到文件 $BINARY"
    exit 1
fi

# 安装到 /usr/local/bin
INSTALL_DIR="/usr/local/bin"
echo "安装 $BINARY 到 $INSTALL_DIR/gitai..."

if [ "$OS" = "Darwin" ] || [ "$OS" = "Linux" ]; then
    sudo cp "$BINARY" "$INSTALL_DIR/gitai"
    sudo chmod +x "$INSTALL_DIR/gitai"
else
    echo "不支持的系统: $OS"
    exit 1
fi

echo "✅ 安装成功！"
echo "运行 'gitai --help' 查看帮助"
EOF

chmod +x "$DIST_DIR/install.sh"

print_msg "📝 已创建安装脚本: $DIST_DIR/install.sh"
