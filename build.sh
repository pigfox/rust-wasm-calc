#!/bin/bash

# Build and Test Script for WASM Calculator
clear
cargo clean
set -e

echo "🦀 Rust WASM Calculator - Build & Test"
echo "======================================"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "⚠️  wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

echo "1️⃣  Running tests..."
echo "===================="
cargo test

echo ""
echo "2️⃣  Building WASM module..."
echo "============================"
wasm-pack build --target web

echo ""
echo "3️⃣  Generating test coverage..."
echo "================================"
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Html --output-dir coverage
    echo "📊 Coverage report generated: coverage/index.html"
else
    echo "⚠️  cargo-tarpaulin not installed. Install with:"
    echo "   cargo install cargo-tarpaulin"
    echo "   Then run: cargo tarpaulin --out Html --output-dir coverage"
fi

echo ""
echo "✅ Build complete!"
echo ""
echo "To run the app:"
echo "  python3 -m http.server 8080"
echo "  Then open: http://localhost:8080"
