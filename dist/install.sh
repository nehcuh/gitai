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
