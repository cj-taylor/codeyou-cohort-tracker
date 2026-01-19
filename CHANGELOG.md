# Changelog

All notable changes to this project will be documented in this file.

Releases are tagged with the date in YYYY.MM.DD format (e.g., v2026.01.19).

## [Unreleased]

### Added
- Self-update functionality with `update` command
- Automatic version checking (checks GitHub for latest release)

### Changed

### Fixed

## [2026.01.19] - 2026-01-19

### Added
- CHANGELOG.md file to track project changes
- GitHub Actions workflows for CI, PR checks, and automated releases
- Dependabot configuration for weekly dependency updates
- Issue templates for bug reports and feature requests
- Pull request template
- Security policy
- Stale bot for inactive issues and PRs
- Multi-platform release binaries (Linux, macOS Intel/ARM, Windows)
- macOS Gatekeeper bypass instructions in release notes

### Changed
- Replaced third-party release action with official gh CLI for more reliable release management
- Enabled vendored OpenSSL for cross-compilation support
- Added explicit permissions to CI workflows (contents: read)

### Fixed
- Release workflow artifact naming to prevent file overwrites
- Incomplete HTML sanitization in JavaScript (XSS vulnerabilities)
- Code formatting issues
- Clippy warning for manual range contains

## [2026.01.15] - 2026-01-15

### Added
- Initial release
- OpenClass API integration for syncing student progress
- SQLite database for local storage
- REST API server
- Interactive dashboard with analytics
- Multi-class support with incremental sync
- Student risk level identification
- Assignment difficulty ranking
- Engagement gap detection
- Progress tracking over time
- Night filtering for cohort comparison
