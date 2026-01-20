# Contributing to ScriptRunner

We love your input! We want to make contributing to ScriptRunner as easy and transparent as possible.

## Development Process

1. Fork the repo
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Make your changes
4. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
5. Push to the branch (`git push origin feature/AmazingFeature`)
6. Open a Pull Request

## Getting Started

### Setup Development Environment

```bash
# Clone your fork
git clone https://github.com/PatrykEmilLibert/script-runner.git
cd script-runner

# Install dependencies
npm install

# Start development
npm run tauri dev
```

### Project Structure

```
src/                 - React components & UI
src-tauri/           - Rust backend
.github/workflows/   - CI/CD automation
examples/            - Example Python scripts
```

### Code Style

**Frontend (TypeScript/React):**
- Use functional components with hooks
- Follow ESLint rules
- Use TypeScript for type safety
- Name components in PascalCase

**Backend (Rust):**
- Follow `cargo fmt` formatting
- Run `cargo clippy` for linting
- Use meaningful variable names
- Add comments for complex logic

## Testing

```bash
# Test frontend
npm run test

# Test backend
cargo test --manifest-path=src-tauri/Cargo.toml

# Build full app
npm run tauri build
```

## Reporting Bugs

Use GitHub Issues to report bugs. Include:
- OS (Windows/Mac version)
- Steps to reproduce
- Expected behavior
- Actual behavior
- Screenshots if applicable
- Logs from `~/.scriptrunner/logs/`

## Feature Requests

Suggest features via GitHub Issues with:
- Use case description
- Why this would be useful
- Possible implementation approach
- Any alternative solutions considered

## Pull Request Process

1. Update the README.md with any new features
2. Update CHANGELOG.md with your changes
3. Add tests for new functionality
4. Ensure all tests pass locally
5. Request reviews from maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to open an issue with the label `question`.
