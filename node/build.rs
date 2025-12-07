//! Build script for ARED Edge Node.
//!
//! This script embeds build information into the node binary.

fn main() {
    substrate_build_script_utils::generate_cargo_keys();
    substrate_build_script_utils::rerun_if_git_head_changed();
}
