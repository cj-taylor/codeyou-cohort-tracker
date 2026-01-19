# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Fixed

## [0.1.5] - 2026-01-19

### Added
- CHANGELOG.md file to track project changes

## [0.1.4] - 2026-01-19

### Changed
- Replaced third-party release action with official gh CLI for more reliable release management

### Fixed
- Release workflow artifact naming to prevent file overwrites

## [0.1.3] - 2026-01-19

### Added
- macOS Gatekeeper bypass instructions in release notes

### Changed
- Added explicit permissions to CI workflows (contents: read)

## [0.1.2] - 2026-01-19

### Fixed
- Release workflow asset preparation to preserve unique artifact names

## [0.1.1] - 2026-01-19

### Added
- GitHub Actions workflows for CI, PR checks, and automated releases
- Dependabot configuration for weekly dependency updates
- Issue templates for bug reports and feature requests
- Pull request template
- Security policy
- Stale bot for inactive issues and PRs
- Multi-platform release binaries (Linux, macOS Intel/ARM, Windows)

### Changed
- Enabled vendored OpenSSL for cross-compilation support

### Fixed
- Incomplete HTML sanitization in JavaScript (XSS vulnerabilities)
- Code formatting issues
- Clippy warning for manual range contains

## [0.1.0] - 2026-01-15

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
