# List the available recipes
default:
  @just --list --justfile {{justfile()}}

# Check that all prerequisites are installed
prereq:
    @echo "Checking prerequisites..."
    @just --version
    @cargo --version
    @cargo fmt --version
    @cargo audit --version
    @cargo clippy --version
    @cargo nextest --version

# Build the project
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the project
run *args:
    cargo run -- {{args}}

# Run tests
test:
    cargo nextest run

# Check code formatting
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run linting (clippy and audit)
lint:
    cargo clippy
    cargo audit

# Clean build artifacts
clean:
    cargo clean
    @rm result >/dev/null 2>&1 || true

# Install the binary locally
install:
    cargo install --path .

# Show cargo version and project info
info:
    cargo --version
    @echo "Project: $(grep '^name' Cargo.toml | cut -d '"' -f 2)"
    @echo "Version: $(grep '^version' Cargo.toml | cut -d '"' -f 2)"

# Development workflow: format, lint, test
dev: fmt lint test

# Full CI workflow: prereq, clean, format, lint, test, build
ci: prereq clean fmt lint test build
