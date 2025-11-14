# Makefile for P-Project Formal Verification

# Default target
.PHONY: all
all: verify

# Install verification tools
.PHONY: install-tools
install-tools:
	cargo install --locked kani-verifier
	cargo kani setup
	@echo "Verification tools installed successfully"

# Run Kani verification
.PHONY: verify-kani
verify-kani:
	cargo kani --features verification

# Run Kani verification with verbose output
.PHONY: verify-kani-verbose
verify-kani-verbose:
	cargo kani --features verification --verbose

# Run Kani verification with specific harness
.PHONY: verify-harness
verify-harness:
	cargo kani --features verification --harness $(HARNESS)

# Run all verification tests
.PHONY: verify
verify: verify-kani

# Clean verification artifacts
.PHONY: clean
clean:
	cargo clean

# Help target
.PHONY: help
help:
	@echo "P-Project Formal Verification Makefile"
	@echo "====================================="
	@echo "Available targets:"
	@echo "  all               - Run all verification (default)"
	@echo "  install-tools     - Install verification tools"
	@echo "  verify            - Run all verification tests"
	@echo "  verify-kani       - Run Kani verification"
	@echo "  verify-kani-verbose - Run Kani verification with verbose output"
	@echo "  verify-harness    - Run specific verification harness (use HARNESS=name)"
	@echo "  clean             - Clean build artifacts"
	@echo "  help              - Show this help message"