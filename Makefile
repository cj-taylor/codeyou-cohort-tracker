.PHONY: build release test test-unit test-integration test-verbose clean init sync status list activate deactivate serve help

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
	@echo "  init      - Initialize configuration (requires EMAIL, PASSWORD)"
	@echo "  list      - List all active classes"
	@echo "  list-all  - List all classes (including inactive)"
	@echo "  activate  - Activate a class (requires CLASS=friendly-id)"
	@echo "  deactivate - Deactivate a class (requires CLASS=friendly-id)"
	@echo "  sync      - Sync data from OpenClass (all active classes)"
	@echo "  sync-class - Sync specific class (requires CLASS=friendly-id)"
	@echo "  status    - Show database status"
	@echo "  serve     - Start server and open dashboard in browser"

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
	@if [ -z "$(EMAIL)" ] || [ -z "$(PASSWORD)" ]; then \
		echo "Usage: make init EMAIL=your@email.com PASSWORD=yourpass"; \
		echo "Note: You will be prompted to select which classes to activate"; \
		exit 1; \
	fi
	cargo run --bin cohort-tracker -- init --email $(EMAIL) --password $(PASSWORD)

list:
	cargo run --bin cohort-tracker -- list

list-all:
	cargo run --bin cohort-tracker -- list --all

activate:
	@if [ -z "$(CLASS)" ]; then \
		echo "Usage: make activate CLASS=friendly-id"; \
		echo "Example: make activate CLASS=data-analysis-pathway-module-1-aug-2"; \
		exit 1; \
	fi
	cargo run --bin cohort-tracker -- activate $(CLASS)

deactivate:
	@if [ -z "$(CLASS)" ]; then \
		echo "Usage: make deactivate CLASS=friendly-id"; \
		exit 1; \
	fi
	cargo run --bin cohort-tracker -- deactivate $(CLASS)

sync:
	cargo run --bin cohort-tracker -- sync

sync-class:
	@if [ -z "$(CLASS)" ]; then \
		echo "Usage: make sync-class CLASS=friendly-id"; \
		exit 1; \
	fi
	cargo run --bin cohort-tracker -- sync --class $(CLASS)

status:
	cargo run --bin cohort-tracker -- status

serve:
	@echo "Starting server on http://localhost:3000/dashboard/"
	@(sleep 2 && open http://localhost:3000/dashboard/) &
	cargo run --bin cohort-tracker -- server
