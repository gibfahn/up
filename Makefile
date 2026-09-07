SHELL := /bin/bash -euo pipefail

# Default target
.DEFAULT_GOAL := help

# To add a target to the help, add a double comment (##) on the target line.
.PHONY: help
help: ## Print help for targets.
	@printf "For more targets and info see the comments in the Makefile.\n\n"
	@grep -E '^[a-zA-Z0-9._-]+:.*?## .*$$' Makefile | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

.PHONY: test
test: ## Run unit and integration tests.
	meta/test

.PHONY: test
bootstrap: ## Run bootstrap tests.
	meta/bootstrap-test

.PHONY: release
release: ## Ship a new release of liv (specify version: `make release bump=minor`).
	meta/release

.PHONY: build-macos
build: ## Build a universal (arm64 + x86_64) release binary.
	meta/build_macos

.PHONY: deps
deps: ## Update cargo dependencies and commit the result.
	meta/update_deps
