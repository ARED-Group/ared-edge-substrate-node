//! Command line interface for ARED Edge node.

use sc_cli::RunCmd;

/// ARED Edge node CLI.
#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// Run the node.
    #[clap(flatten)]
    pub run: RunCmd,
}

/// Available subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    #[cfg(feature = "runtime-benchmarks")]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Key management commands.
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
