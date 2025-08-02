.PHONY: help build test bench clean run install check fmt clippy doc

# Default target
help:
	@echo "Ferrum Minecraft Client - Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build      - Build all crates in release mode"
	@echo "  make test       - Run all tests"
	@echo "  make bench      - Run benchmarks"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make run        - Run the client"
	@echo "  make install    - Install dependencies"
	@echo "  make check      - Run cargo check"
	@echo "  make fmt        - Format code"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make doc        - Generate documentation"

# Build in release mode
build:
	RUSTC_BOOTSTRAP=1 cargo build --release --workspace

# Run all tests
test:
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-physics
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-world
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-inventory
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-meshing-cpu
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-assets
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-config
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-protocol
	RUSTC_BOOTSTRAP=1 cargo test --package ferrum-subprocess

# Run benchmarks
bench:
	RUSTC_BOOTSTRAP=1 cargo bench --package ferrum-meshing-cpu

# Clean build artifacts
clean:
	cargo clean

# Run the client
run:
	RUSTC_BOOTSTRAP=1 cargo run --release

# Install system dependencies (Linux)
install:
	@echo "Installing system dependencies..."
	@if command -v apt-get > /dev/null; then \
		sudo apt-get update && sudo apt-get install -y build-essential pkg-config libx11-dev libasound2-dev libudev-dev; \
	elif command -v pacman > /dev/null; then \
		sudo pacman -S base-devel alsa-lib; \
	elif command -v dnf > /dev/null; then \
		sudo dnf install gcc pkg-config libX11-devel alsa-lib-devel systemd-devel; \
	else \
		echo "Unsupported package manager. Please install dependencies manually."; \
	fi

# Run cargo check
check:
	RUSTC_BOOTSTRAP=1 cargo check --workspace

# Format code
fmt:
	cargo fmt --all

# Run clippy
clippy:
	RUSTC_BOOTSTRAP=1 cargo clippy --workspace -- -D warnings

# Generate documentation
doc:
	RUSTC_BOOTSTRAP=1 cargo doc --workspace --no-deps --open
