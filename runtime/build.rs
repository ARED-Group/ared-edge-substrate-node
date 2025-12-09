//! ARED Edge Runtime Build Script
//!
//! Compiles the runtime to WASM using substrate-wasm-builder.
//! Uses the same configuration as the official Polkadot SDK solochain template.

#[cfg(feature = "std")]
fn main() {
    substrate_wasm_builder::WasmBuilder::build_using_defaults();
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}
