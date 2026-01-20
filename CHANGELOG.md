# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-01-20

### Added
- Initial release of ScriptRunner
- Desktop app built with Tauri + React
- Python script execution with auto-dependency detection
- Automatic script syncing from GitHub
- Remote kill switch for emergency lockdown
- Dark mode UI with modern design
- Support for Windows and Mac
- Kill switch implementation
- AST-based dependency parsing
- Example Python scripts
- Comprehensive documentation

### Features
- **Auto-Update**: App and scripts update from GitHub
- **Dependency Detection**: Automatic `import` analysis using AST
- **Kill Switch**: Remote block via GitHub JSON
- **Logs**: Real-time output capture and history
- **Modern UI**: React 18 + TailwindCSS + Framer Motion

### Technical
- Tauri 1.x for lightweight desktop app
- React 18 with TypeScript
- Rust backend for system integration
- GitHub Actions for CI/CD

## [Unreleased]

### Planned Features
- User authentication
- Script scheduling
- Advanced logging & monitoring
- Environment variable management
- Script marketplace
- Plugin system
- Docker support
- API server mode
- Web UI alternative
