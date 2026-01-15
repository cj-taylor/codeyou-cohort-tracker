# Contributing to Cohort Tracker

Thanks for your interest in contributing! This project is built to help educators track student progress, and we welcome improvements.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/codeyou-cohort-tracker.git`
3. Create a branch: `git checkout -b your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Format code: `cargo fmt`
7. Check lints: `cargo clippy -- -D warnings`
8. Commit your changes
9. Push to your fork
10. Open a pull request

## Development Setup

See the [Development Guide](./docs/development.md) for detailed setup instructions.

## Code Quality

All pull requests must pass:
- Build: `cargo build`
- Tests: `cargo test`
- Formatting: `cargo fmt -- --check`
- Linting: `cargo clippy -- -D warnings`

These checks run automatically via GitHub Actions on every PR.

## Pull Request Guidelines

- Keep changes focused and atomic
- Write clear commit messages
- Add tests for new features
- Update documentation as needed
- Follow existing code style

## Questions?

- Check the [documentation](./docs/README.md)
- Open an issue for discussion
- Look at existing code for examples

## Code of Conduct

Be respectful and constructive. This is a learning-focused project.
