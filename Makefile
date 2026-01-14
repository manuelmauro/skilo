.PHONY: setup clean fmt-check fmt clippy clippy-release check check-release build build-release test test-release install lint skill-fmt new-skill ci doc help

# Setup development environment
setup:
	rustup component add rustfmt clippy
	cargo fetch

# Cleanup compilation outputs
clean:
	cargo clean

# Check the code format
fmt-check:
	cargo fmt --all -- --check
# Format the code
fmt:
	cargo fmt --all

# Run rust clippy with debug profile
clippy:
	cargo clippy --all --all-targets -- -D warnings
# Run rust clippy with release profile
clippy-release:
	cargo clippy --release --all --all-targets -- -D warnings

# Check code with debug profile
check:
	cargo check
# Check code with release profile
check-release:
	cargo check --release

# Build all binaries with debug profile
build:
	cargo build
# Build all binaries with release profile
build-release:
	cargo build --release

# Run all unit tests with debug profile
test:
	cargo test --all
# Run all unit tests with release profile
test-release:
	cargo test --release --all

# Install the binary locally
install:
	cargo install --path .

# Run skilo lint on skills
lint:
	cargo run -- lint .

# Format skills
skill-fmt:
	cargo run -- fmt .

# Create a new test skill (usage: make new-skill NAME=my-skill LANG=python)
new-skill:
	cargo run -- new $(NAME) --lang $(or $(LANG),python)

# Run all CI checks (fmt, clippy, test, build, lint, skill-fmt)
ci: fmt clippy test build lint skill-fmt

# Generate documentation
doc:
	cargo doc --no-deps --open

# Show help
help:
	@echo ''
	@echo 'Usage:'
	@echo ' make [target]'
	@echo ''
	@echo 'Targets:'
	@awk '/^[a-zA-Z\-\_0-9]+:/ { \
	helpMessage = match(lastLine, /^# (.*)/); \
		if (helpMessage) { \
			helpCommand = substr($$1, 0, index($$1, ":")); \
			helpMessage = substr(lastLine, RSTART + 2, RLENGTH); \
			printf "\033[36m%-30s\033[0m %s\n", helpCommand,helpMessage; \
		} \
	} \
	{ lastLine = $$0 }' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help
