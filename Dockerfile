# =============================================================================
# ARED Edge Substrate Node - Dockerfile
# =============================================================================
# Multi-stage build for Substrate blockchain node
# =============================================================================

# -----------------------------------------------------------------------------
# Build Stage
# -----------------------------------------------------------------------------
FROM rust:1.79-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    clang \
    libclang-dev \
    llvm \
    cmake \
    protobuf-compiler \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /build

# Copy manifests first for better layer caching
COPY Cargo.toml ./
COPY Cargo.lock* ./
COPY node/Cargo.toml ./node/
COPY runtime/Cargo.toml ./runtime/
COPY pallets/ ./pallets/

# Create dummy source files for dependency compilation
RUN mkdir -p node/src runtime/src && \
    echo "fn main() {}" > node/src/main.rs && \
    echo "" > runtime/src/lib.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release --package ared-edge-node || true

# Copy actual source code
COPY . .

# Build the actual binary
RUN cargo build --release --package ared-edge-node

# -----------------------------------------------------------------------------
# Runtime Stage
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd --gid 1000 substrate && \
    useradd --uid 1000 --gid substrate --shell /bin/bash --create-home substrate

# Copy binary from builder
COPY --from=builder /build/target/release/ared-edge-node /usr/local/bin/

# Create data directory
RUN mkdir -p /data && chown substrate:substrate /data

# Labels
LABEL org.opencontainers.image.title="ARED Edge Substrate Node" \
      org.opencontainers.image.description="Private blockchain node for ARED Edge IoT Platform" \
      org.opencontainers.image.vendor="ARED"

# Switch to non-root user
USER substrate

# Data volume
VOLUME ["/data"]

# Expose ports
# 30333 - P2P
# 9944  - RPC/WebSocket
# 9615  - Prometheus metrics
EXPOSE 30333 9944 9615

# Health check using RPC system_health endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -sf -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"system_health"}' \
    http://localhost:9944 | grep -q '"isSyncing"' || exit 1

# Default entrypoint
ENTRYPOINT ["ared-edge-node"]
CMD ["--dev"]
