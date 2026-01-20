.PHONY: help install dev build release clean

help:
	@echo "ScriptRunner - Make Commands"
	@echo ""
	@echo "install      - Install all dependencies"
	@echo "dev          - Start development server with hot reload"
	@echo "build        - Build frontend"
	@echo "tauri-dev    - Run Tauri dev (full desktop app)"
	@echo "tauri-build  - Build production executable"
	@echo "clean        - Clean build artifacts"
	@echo "release      - Create production release"

install:
	npm install
	cargo build --manifest-path=src-tauri/Cargo.toml

dev:
	npm run dev

tauri-dev:
	npm run tauri dev

build:
	npm run build

tauri-build:
	npm run tauri build

clean:
	rm -rf dist
	rm -rf src-tauri/target
	cargo clean --manifest-path=src-tauri/Cargo.toml

release: clean build tauri-build
	@echo "Build complete! Check src-tauri/target/release/ for output"
