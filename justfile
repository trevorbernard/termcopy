# List the available recipes
default:
  @just --list --justfile {{justfile()}}

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
    cargo test

# Check code formatting
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy linter
clippy:
    cargo clippy

# Clean build artifacts
clean:
    cargo clean
    rm result >/dev/null 2>&1 || true

# Install the binary locally
install:
    cargo install --path .

# Show cargo version and project info
info:
    cargo --version
    @echo "Project: $(grep '^name' Cargo.toml | cut -d '"' -f 2)"
    @echo "Version: $(grep '^version' Cargo.toml | cut -d '"' -f 2)"

# Development workflow: format, check, test
dev: fmt clippy test

# Full CI workflow: clean, format, check, test, build
ci: clean fmt clippy test build
