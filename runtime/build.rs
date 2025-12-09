//! ARED Edge Runtime Build Script
//!
//! This build script compiles the runtime to WASM using substrate-wasm-builder.

#[cfg(feature = "std")]
fn main() {
    substrate_wasm_builder::WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .build();
}

#[cfg(not(feature = "std"))]
fn main() {}
