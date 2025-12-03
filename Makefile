# =============================================================================
# ARED Edge Substrate Node - Makefile
# =============================================================================

.PHONY: help build build-release test lint format check docker-build \
        run-dev deploy-dev deploy-prod clean

SHELL := /bin/bash
.DEFAULT_GOAL := help

# Configuration
PROJECT_NAME := ared-edge-substrate-node
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
DOCKER_REGISTRY ?= ghcr.io/ared
DOCKER_TAG ?= $(VERSION)

# Colors
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

help: ## Show this help
	@echo ""
	@echo "$(CYAN)ARED Edge Substrate Node$(RESET)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@echo ""

# -----------------------------------------------------------------------------
# Development
# -----------------------------------------------------------------------------
setup: ## Install Rust toolchain and dependencies
	@echo "$(CYAN)Setting up development environment...$(RESET)"
	rustup update stable
	rustup target add wasm32-unknown-unknown
	cargo install cargo-watch cargo-audit
	@echo "$(GREEN)Setup complete$(RESET)"

build: ## Build debug version
	@echo "$(CYAN)Building debug...$(RESET)"
	cargo build

build-release: ## Build release version
	@echo "$(CYAN)Building release...$(RESET)"
	cargo build --release

build-runtime: ## Build WASM runtime
	@echo "$(CYAN)Building WASM runtime...$(RESET)"
	cargo build --release -p ared-edge-runtime

test: ## Run all tests
	@echo "$(CYAN)Running tests...$(RESET)"
	cargo test --all

test-runtime: ## Run runtime tests
	cargo test -p ared-edge-runtime

bench: ## Run benchmarks
	@echo "$(CYAN)Running benchmarks...$(RESET)"
	cargo build --release --features runtime-benchmarks

# -----------------------------------------------------------------------------
# Code Quality
# -----------------------------------------------------------------------------
lint: ## Run clippy linter
	@echo "$(CYAN)Running clippy...$(RESET)"
	cargo clippy --all-targets --all-features -- -D warnings

format: ## Format code
	@echo "$(CYAN)Formatting code...$(RESET)"
	cargo fmt --all

format-check: ## Check formatting
	cargo fmt --all -- --check

check: format-check lint test ## Run all checks
	@echo "$(GREEN)All checks passed$(RESET)"

audit: ## Security audit dependencies
	@echo "$(CYAN)Auditing dependencies...$(RESET)"
	cargo audit

# -----------------------------------------------------------------------------
# Running
# -----------------------------------------------------------------------------
run-dev: ## Run development node
	@echo "$(CYAN)Starting dev node...$(RESET)"
	cargo run --release -- --dev --tmp

run-alice: ## Run Alice validator
	cargo run --release -- \
		--chain=local \
		--alice \
		--base-path=/tmp/alice \
		--port=30333 \
		--rpc-port=9944 \
		--validator

run-bob: ## Run Bob validator
	cargo run --release -- \
		--chain=local \
		--bob \
		--base-path=/tmp/bob \
		--port=30334 \
		--rpc-port=9945 \
		--validator

# -----------------------------------------------------------------------------
# Docker
# -----------------------------------------------------------------------------
docker-build: ## Build Docker image
	@echo "$(CYAN)Building Docker image...$(RESET)"
	docker build -t $(DOCKER_REGISTRY)/substrate-node:$(DOCKER_TAG) .

docker-push: ## Push Docker image
	docker push $(DOCKER_REGISTRY)/substrate-node:$(DOCKER_TAG)

docker-run: ## Run Docker container
	docker run -it --rm \
		-p 9944:9944 -p 9933:9933 -p 30333:30333 \
		$(DOCKER_REGISTRY)/substrate-node:$(DOCKER_TAG) --dev

# -----------------------------------------------------------------------------
# Kubernetes
# -----------------------------------------------------------------------------
deploy-dev: ## Deploy to development
	@echo "$(CYAN)Deploying to development...$(RESET)"
	kubectl apply -k k8s/dev/ -n ared-edge

deploy-prod: ## Deploy to production
	@echo "$(YELLOW)Deploying to production...$(RESET)"
	kubectl apply -k k8s/prod/ -n ared-edge

# -----------------------------------------------------------------------------
# Cleanup
# -----------------------------------------------------------------------------
clean: ## Clean build artifacts
	@echo "$(CYAN)Cleaning...$(RESET)"
	cargo clean
	rm -rf data/ chains/

purge-chain: ## Purge chain data
	@echo "$(YELLOW)Purging chain data...$(RESET)"
	./target/release/ared-edge-node purge-chain --dev -y
