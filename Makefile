.PHONY: build release test test-unit test-integration test-verbose clean init sync status list activate deactivate serve help check fmt clippy

# Default target
help:
	@echo "Cohort Tracker - Available commands:"
	@echo ""
	@echo "Building:"
	@echo "  make build     - Build debug version (cargo build)"
	@echo "  make release   - Build optimized release version (cargo build --release)"
	@echo "  make check     - Quick compile check without building (cargo check)"
	@echo ""
	@echo "Testing:"
	@echo "  make test      - Run all tests (cargo test)"
	@echo "  make test-unit - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make test-verbose - Run tests with output"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt       - Format code (cargo fmt)"
	@echo "  make clippy    - Run linter (cargo clippy)"
	@echo ""
	@echo "Setup:"
	@echo "  make init EMAIL=you@example.com PASSWORD=pass"
	@echo "                 - Initialize with OpenClass credentials"
	@echo ""
	@echo "Daily Usage:"
	@echo "  make sync      - Sync all active classes (incremental)"
	@echo "  make sync-full - Full sync (fetches everything)"
	@echo "  make status    - Show what's tracked and last sync time"
	@echo "  make serve     - Start dashboard at http://localhost:3000"
	@echo ""
	@echo "Class Management:"
	@echo "  make list      - List active classes"
	@echo "  make list-all  - List all classes (including inactive)"
	@echo "  make activate CLASS=friendly-id"
	@echo "  make deactivate CLASS=friendly-id"
	@echo "  make sync-class CLASS=friendly-id"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean     - Remove build artifacts (cargo clean)"
	@echo ""
	@echo "Examples:"
	@echo "  make init EMAIL=mentor@example.com PASSWORD=secret123"
	@echo "  make sync"
	@echo "  make serve"
	@echo "  make sync-class CLASS=data-analysis-pathway-module-2-aug-2"

# Build commands
build:
	@echo "Building debug version..."
	cargo build

release:
	@echo "Building optimized release version..."
	cargo build --release

check:
	@echo "Checking code (fast compile check)..."
	cargo check

# Code quality
fmt:
	@echo "Formatting code..."
	cargo fmt

clippy:
	@echo "Running linter..."
	cargo clippy

# Testing
test:
	@echo "Running all tests..."
	cargo test

test-unit:
	@echo "Running unit tests..."
	cargo test --lib

test-integration:
	@echo "Running integration tests..."
	cargo test --test '*'

test-verbose:
	@echo "Running tests with output..."
	cargo test -- --nocapture

# Cleanup
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Application commands
init:
	@if [ -z "$(EMAIL)" ] || [ -z "$(PASSWORD)" ]; then \
		echo "Usage: make init EMAIL=your@email.com PASSWORD=yourpass"; \
		echo ""; \
		echo "This will:"; \
		echo "  1. Save your credentials to ~/.cohort-tracker.toml"; \
		echo "  2. Fetch all classes you have access to"; \
		echo "  3. Let you select which classes to track"; \
		echo ""; \
		echo "Example: make init EMAIL=mentor@example.com PASSWORD=secret123"; \
		exit 1; \
	fi
	@echo "Initializing with OpenClass credentials..."
	cargo run --bin cohort-tracker -- init --email $(EMAIL) --password $(PASSWORD)

list:
	@echo "Listing active classes..."
	cargo run --bin cohort-tracker -- list

list-all:
	@echo "Listing all classes..."
	cargo run --bin cohort-tracker -- list --all

activate:
	@if [ -z "$(CLASS)" ]; then \
		echo "Usage: make activate CLASS=friendly-id"; \
		echo ""; \
		echo "Example: make activate CLASS=data-analysis-pathway-module-1-aug-2"; \
		echo ""; \
		echo "Tip: Run 'make list-all' to see available class IDs"; \
		exit 1; \
	fi
	@echo "Activating class: $(CLASS)"
	cargo run --bin cohort-tracker -- activate $(CLASS)

deactivate:
	@if [ -z "$(CLASS)" ]; then \
		echo "Usage: make deactivate CLASS=friendly-id"; \
		echo ""; \
		echo "Example: make deactivate CLASS=old-class-name"; \
		exit 1; \
	fi
	@echo "Deactivating class: $(CLASS)"
	cargo run --bin cohort-tracker -- deactivate $(CLASS)

sync:
	@echo "Syncing all active classes (incremental)..."
	cargo run --bin cohort-tracker -- sync

sync-full:
	@echo "Full sync (fetches everything)..."
	cargo run --bin cohort-tracker -- sync --full

sync-class:
	@if [ -z "$(CLASS)" ]; then \
		echo "Usage: make sync-class CLASS=friendly-id"; \
		echo ""; \
		echo "Example: make sync-class CLASS=data-analysis-pathway-module-2-aug-2"; \
		exit 1; \
	fi
	@echo "Syncing class: $(CLASS)"
	cargo run --bin cohort-tracker -- sync --class $(CLASS)

status:
	@echo "Checking status..."
	cargo run --bin cohort-tracker -- status

serve:
	@echo "Starting server on http://localhost:3000"
	@echo "Dashboard will open in your browser..."
	@(sleep 2 && open http://localhost:3000 2>/dev/null || xdg-open http://localhost:3000 2>/dev/null || echo "Open http://localhost:3000 in your browser") &
	cargo run --bin cohort-tracker -- server
