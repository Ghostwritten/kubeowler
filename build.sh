#!/bin/bash

# Kubeowler 编译和运行脚本

set -e

echo "🔍 Kubeowler - Kubernetes 集群巡检工具"
echo "========================================="

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到 Cargo (Rust 包管理器)"
    echo ""
    echo "请先安装 Rust:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "source ~/.cargo/env"
    exit 1
fi

echo "✅ 找到 Rust 环境"
echo "   - Rust 版本: $(rustc --version)"
echo "   - Cargo 版本: $(cargo --version)"
echo ""

# 检查项目文件
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: 未找到 Cargo.toml 文件"
    echo "请确保在项目根目录下运行此脚本"
    exit 1
fi

echo "✅ 项目文件检查通过"
echo ""

# 编译项目
echo "🔧 开始编译项目..."
if [ "$1" = "--release" ]; then
    echo "   编译模式: 优化版本 (release)"
    cargo build --release
    BINARY_PATH="./target/release/kubeowler"
else
    echo "   编译模式: 开发版本 (debug)"
    cargo build
    BINARY_PATH="./target/debug/kubeowler"
fi

if [ $? -eq 0 ]; then
    echo "✅ 编译成功!"
    echo ""

    # 显示二进制文件信息
    if [ -f "$BINARY_PATH" ]; then
        echo "📦 二进制文件信息:"
        echo "   路径: $BINARY_PATH"
        echo "   大小: $(du -h $BINARY_PATH | cut -f1)"
        echo ""

        # 显示使用示例
        echo "🚀 使用示例:"
        echo "   # 显示帮助"
        echo "   $BINARY_PATH check --help"
        echo ""
        echo "   # 全集群巡检"
        echo "   $BINARY_PATH check"
        echo ""
        echo "   # 指定命名空间"
        echo "   $BINARY_PATH check -n kube-system"
        echo ""
        echo "   # 自定义输出文件与格式"
        echo "   $BINARY_PATH check -o my-report.md"
        echo "   $BINARY_PATH check -o report.json -f json"
        echo ""
    fi
else
    echo "❌ 编译失败"
    exit 1
fi

# 运行测试
if [ "$2" = "--test" ]; then
    echo "🧪 运行测试..."
    cargo test
    if [ $? -eq 0 ]; then
        echo "✅ 所有测试通过!"
    else
        echo "❌ 部分测试失败"
    fi
fi

echo "🎉 准备完成! 现在可以使用 Kubeowler 进行集群巡检了。"