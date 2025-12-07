//! ARED Edge Node Build Script
//!
//! Sets the SUBSTRATE_CLI_IMPL_VERSION environment variable for the node binary.

fn main() {
    substrate_build_script_utils::generate_cargo_keys();
    substrate_build_script_utils::rerun_if_git_head_changed();
}
