# Contributing to ARED Edge Substrate Node

## Development Setup

### Prerequisites
- Rust 1.74+ (with `wasm32-unknown-unknown` target)
- Docker and Docker Compose
- Make

### Quick Start

```bash
# Install Rust toolchain
rustup update stable
rustup target add wasm32-unknown-unknown

# Build the node
cargo build --release

# Run tests
cargo test --all

# Run local dev chain
./target/release/ared-edge-node --dev
```

## Coding Standards

### Rust Style
- Follow Rust API guidelines
- Use `cargo fmt` before committing
- Run `cargo clippy` and address all warnings
- Document all public APIs

### Pallet Development
- Each pallet in its own crate under `/pallets`
- Include benchmarks for all extrinsics
- Write comprehensive unit tests
- Document storage items and events

## Commit Guidelines

Follow Conventional Commits:
```
feat(pallet-carbon): add carbon credit calculation
fix(runtime): correct weight calculation
docs(node): update RPC documentation
```

## Testing Requirements

- Unit tests: `cargo test`
- Integration tests: `cargo test --features runtime-benchmarks`
- Benchmark: `cargo build --release --features runtime-benchmarks`

## Pull Request Process

1. Create feature branch from `main`
2. Write tests for new functionality
3. Update documentation
4. Run `cargo fmt` and `cargo clippy`
5. Submit PR with clear description
