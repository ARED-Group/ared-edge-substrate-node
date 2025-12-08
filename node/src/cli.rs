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

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Key management commands.
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
