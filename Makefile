.PHONY: build release test test-unit test-integration test-verbose clean init sync status help

# Default target
help:
	@echo "Cohort Tracker - Available commands:"
	@echo "  build     - Build debug version"
	@echo "  release   - Build optimized release version"
	@echo "  test      - Run all tests"
	@echo "  test-unit - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-verbose - Run tests with output"
	@echo "  clean     - Clean build artifacts"
	@echo "  init      - Initialize configuration (requires EMAIL, PASSWORD, CLASS_ID)"
	@echo "  sync      - Sync data from OpenClass"
	@echo "  status    - Show database status"

# Build commands
build:
	cargo build

release:
	cargo build --release

# Testing
test:
	cargo test

test-unit:
	cargo test --lib

test-integration:
	cargo test --test '*'

test-verbose:
	cargo test -- --nocapture

# Cleanup
clean:
	cargo clean

# Application commands
init:
	@if [ -z "$(EMAIL)" ] || [ -z "$(PASSWORD)" ] || [ -z "$(CLASS_ID)" ]; then \
		echo "Usage: make init EMAIL=your@email.com PASSWORD=yourpass CLASS_ID=68e594f320442cbbe62a18dc"; \
		exit 1; \
	fi
	cargo run -- init --email $(EMAIL) --password $(PASSWORD) --class-id $(CLASS_ID)

sync:
	cargo run -- sync

status:
	cargo run -- status
