BIN := jlic
CARGO ?= cargo
DIST := dist
VERSION := $(shell awk -F'"' '/^version/ {print $$2; exit}' Cargo.toml)

.DEFAULT_GOAL := help

.PHONY: help
help: ## Show the target list
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: build
build: ## Build with the debug profile
	$(CARGO) build

.PHONY: release
release: ## Build with the release profile
	$(CARGO) build --release

.PHONY: run
run: ## Run it: make run ARGS="mit --stdout"
	$(CARGO) run -- $(ARGS)

.PHONY: test
test: ## Run every test
	$(CARGO) test --all-features

.PHONY: fmt
fmt: ## Format the code
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Verify the code is formatted
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint: ## Clippy with warnings denied
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: check
check: fmt-check lint test ## Full check, same as CI

.PHONY: audit
audit: ## Scan dependencies for known vulnerabilities
	@command -v cargo-audit >/dev/null 2>&1 || { \
		echo "cargo-audit is not installed: cargo install --locked cargo-audit"; \
		exit 1; \
	}
	$(CARGO) audit --deny warnings

.PHONY: doc
doc: ## Build and open the API documentation
	$(CARGO) doc --no-deps --open

.PHONY: install
install: ## Install the binary into ~/.cargo/bin
	$(CARGO) install --path . --locked

.PHONY: uninstall
uninstall: ## Remove the installed binary
	$(CARGO) uninstall $(BIN)

.PHONY: completions
completions: release ## Generate shell completions into $(DIST)/completions
	@mkdir -p $(DIST)/completions
	@for sh in bash zsh fish; do \
		./target/release/$(BIN) completions $$sh > $(DIST)/completions/$(BIN).$$sh; \
	done
	@echo "completions → $(DIST)/completions"

.PHONY: man
man: release ## Generate the man page into $(DIST)/man
	@mkdir -p $(DIST)/man
	@./target/release/$(BIN) man > $(DIST)/man/$(BIN).1
	@echo "man page → $(DIST)/man/$(BIN).1"

.PHONY: dist
dist: release completions man ## Build a local archive to inspect a release
	@cp target/release/$(BIN) $(DIST)/
	@cp README.md LICENSE $(DIST)/
	@tar -czf $(DIST)/$(BIN)_$(VERSION)_local.tar.gz -C $(DIST) $(BIN) README.md LICENSE completions man
	@echo "archive → $(DIST)/$(BIN)_$(VERSION)_local.tar.gz"

.PHONY: update-templates
update-templates: ## Refetch license texts and SPDX references
	./scripts/update-templates.sh

.PHONY: publish-dry
publish-dry: ## Verify the crates.io package without publishing
	$(CARGO) publish --dry-run --locked

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean
	rm -rf $(DIST)
