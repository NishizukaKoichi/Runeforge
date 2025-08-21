# Contributing to Runeforge

Thank you for your interest in contributing to Runeforge! This document provides guidelines for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Issues

- Check if the issue already exists
- Use a clear and descriptive title
- Include steps to reproduce the issue
- Provide environment details (OS, Rust version, etc.)

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Ensure all tests pass (`cargo test`)
5. Run formatting and linting:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   ```
6. Commit your changes with a descriptive message
7. Push to your fork
8. Open a Pull Request

### Development Setup

1. Install Rust 1.82+ (MSRV)
2. Clone the repository:
   ```bash
   git clone https://github.com/NishizukaKoichi/Runeforge.git
   cd Runeforge
   ```
3. Run tests:
   ```bash
   cargo test
   ```

### Testing

- Write tests for new functionality
- Ensure existing tests pass
- Add integration tests for new features
- Include test fixtures when appropriate

### Documentation

- Update README.md for user-facing changes
- Add rustdoc comments for public APIs
- Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/)

### Code Style

- Follow Rust standard style guidelines
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Keep functions focused and small
- Write descriptive variable and function names

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters
- Reference issues and pull requests

### Performance

- Benchmark performance-critical code
- Avoid premature optimization
- Document performance characteristics

### Security

- Review dependencies carefully
- Run `cargo audit` before submitting
- Report security vulnerabilities privately

## Release Process

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create a git tag
4. CI will automatically build and release

## Questions?

Feel free to open an issue for any questions about contributing.