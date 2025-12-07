//! ARED Edge Runtime Build Script
//!
//! This build script compiles the runtime to WASM using substrate-wasm-builder.

fn main() {
    substrate_wasm_builder::WasmBuilder::build_using_defaults();
}
