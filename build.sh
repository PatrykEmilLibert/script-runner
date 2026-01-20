#!/bin/bash
# Build script for macOS/Linux

cd "$(dirname "$0")"

echo "Cleaning previous build..."
cd src-tauri
cargo clean
cd ..

echo "Building Tauri app..."
npm run tauri build

echo "Build complete! Check src-tauri/target/release/bundle/"
