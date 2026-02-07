#!/bin/bash

# Kubeowler build and run script

set -e

echo "🔍 Kubeowler - Kubernetes Cluster Checker"
echo "========================================="

# Check Rust environment
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo (Rust package manager) not found"
    echo ""
    echo "Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "source ~/.cargo/env"
    exit 1
fi

echo "✅ Rust environment found"
echo "   - Rust version: $(rustc --version)"
echo "   - Cargo version: $(cargo --version)"
echo ""

# Check project files
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found"
    echo "Please run this script from the project root directory"
    exit 1
fi

echo "✅ Project files OK"
echo ""

# Build project
echo "🔧 Building project..."
if [ "$1" = "--release" ]; then
    echo "   Build mode: release"
    cargo build --release
    BINARY_PATH="./target/release/kubeowler"
else
    echo "   Build mode: debug"
    cargo build
    BINARY_PATH="./target/debug/kubeowler"
fi

if [ $? -eq 0 ]; then
    echo "✅ Build succeeded!"
    echo ""

    # Show binary info
    if [ -f "$BINARY_PATH" ]; then
        echo "📦 Binary:"
        echo "   Path: $BINARY_PATH"
        echo "   Size: $(du -h $BINARY_PATH | cut -f1)"
        echo ""

        # Usage examples
        echo "🚀 Usage examples:"
        echo "   # Show help"
        echo "   $BINARY_PATH check --help"
        echo ""
        echo "   # Full cluster check"
        echo "   $BINARY_PATH check"
        echo ""
        echo "   # Specific namespace"
        echo "   $BINARY_PATH check -n kube-system"
        echo ""
        echo "   # Custom output file and format"
        echo "   $BINARY_PATH check -o my-report.md"
        echo "   $BINARY_PATH check -o report.json -f json"
        echo ""
    fi
else
    echo "❌ Build failed"
    exit 1
fi

# Run tests
if [ "$2" = "--test" ]; then
    echo "🧪 Running tests..."
    cargo test
    if [ $? -eq 0 ]; then
        echo "✅ All tests passed!"
    else
        echo "❌ Some tests failed"
    fi
fi

echo "🎉 Done! You can now run Kubeowler to check your cluster."
