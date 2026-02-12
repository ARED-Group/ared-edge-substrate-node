# syntax=docker/dockerfile:1.4
# =============================================================================
# ARED Edge Substrate Node - Dockerfile
# =============================================================================
# Enterprise-grade multi-stage build with BuildKit optimizations
# - Cargo registry/git caching for faster rebuilds
# - Controlled parallelism to prevent resource exhaustion
# - Separate dev and production build profiles
# =============================================================================

# -----------------------------------------------------------------------------
# Build Stage
# -----------------------------------------------------------------------------
FROM rust:1.85-slim-bookworm AS builder

# Build arguments for controlling resource usage
ARG CARGO_BUILD_JOBS=2
ARG CARGO_PROFILE=release

# Environment variables for build optimization
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS} \
    CARGO_INCREMENTAL=0 \
    CARGO_NET_GIT_FETCH_WITH_CLI=true \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
    RUSTFLAGS="-C codegen-units=16"

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
    perl \
    && rm -rf /var/lib/apt/lists/*

# Add WASM target and rust-src for substrate-wasm-builder
RUN rustup target add wasm32-unknown-unknown && \
    rustup component add rust-src

WORKDIR /build

# Copy manifests, lockfile and build scripts for reproducible builds
COPY Cargo.toml Cargo.lock ./
COPY .cargo/config.toml ./.cargo/
COPY node/Cargo.toml node/build.rs ./node/
COPY runtime/Cargo.toml runtime/build.rs ./runtime/
COPY pallets/ ./pallets/

# Create dummy source files for dependency compilation
RUN mkdir -p node/src runtime/src && \
    echo "fn main() {}" > node/src/main.rs && \
    echo "#![cfg_attr(not(feature = \"std\"), no_std)]" > runtime/src/lib.rs

# Build dependencies only with BuildKit cache mounts
# This dramatically speeds up rebuilds by caching:
# - Cargo registry index and crate sources
# - Cargo git checkouts
# - Compiled dependencies in target directory
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/build/target,sharing=locked \
    cargo build --${CARGO_PROFILE} --package ared-edge-node || true

# Copy actual source code
COPY . .

# Build the actual binary with BuildKit cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/build/target,sharing=locked \
    cargo build --${CARGO_PROFILE} --package ared-edge-node && \
    cp /build/target/${CARGO_PROFILE}/ared-edge-node /build/ared-edge-node

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

# Copy binary from builder (uses the copied binary from build stage)
COPY --from=builder /build/ared-edge-node /usr/local/bin/

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

# Health check using Substrate RPC system_health method
HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=3 \
    CMD curl -sf -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' \
        http://localhost:9944 || exit 1

# Default entrypoint
ENTRYPOINT ["ared-edge-node"]

# Default command for development mode
# For production, override with: --chain=/data/chainspec.json --base-path=/data
# RPC options: --rpc-external --rpc-cors=all --rpc-methods=safe
CMD ["--dev", "--base-path=/data", "--rpc-external", "--rpc-cors=all", "--prometheus-external"]
