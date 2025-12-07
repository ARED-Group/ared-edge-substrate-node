//! Build script for ARED Edge Runtime.
//!
//! This script compiles the runtime to WASM for on-chain execution.

fn main() {
    substrate_wasm_builder::WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .build();
}
