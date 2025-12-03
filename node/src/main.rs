//! ARED Edge Substrate Node
//!
//! This is the main entry point for the ARED Edge private blockchain node.
//! The node provides:
//! - Consensus participation (Aura + GRANDPA)
//! - RPC endpoints for extrinsics and state queries
//! - Block production and validation
//! - Event emission for downstream consumers

#![warn(missing_docs)]

mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
    command::run()
}
