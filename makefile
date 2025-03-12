.PHONY: build build-release test clean doc clippy fmt check all

# Default target
all: build

# Build the library
build:
	cargo build --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Generate documentation
doc:
	cargo doc --no-deps

# Run linter
clippy:
	cargo clippy -- -D warnings

# Run code formatter
fmt:
	cargo fmt

# Run various checks (format, clippy, test)
check: fmt clippy test

# Install the library to the local cargo registry
install:
	cargo install --path .

# Watch for changes and rebuild
watch:
	cargo watch -x build -x test
