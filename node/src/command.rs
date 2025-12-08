//! ARED Edge Node Command Handler
//!
//! Processes CLI commands and starts the appropriate node mode.

use crate::chain_spec;
use crate::cli::{Cli, Subcommand};
use crate::service;
use sc_cli::SubstrateCli;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "ARED Edge Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        "ARED Edge private blockchain node for IoT telemetry proofs and carbon credits".into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/ared/ared-edge-substrate-node/issues".into()
    }

    fn copyright_start_year() -> i32 {
        2024
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" | "development" => Box::new(chain_spec::development_config()?),
            "local" | "local_testnet" => Box::new(chain_spec::local_testnet_config()?),
            "" | "prod" | "production" => Box::new(chain_spec::production_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }
}

/// Run the CLI.
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let partial = service::new_partial(&config)?;
                Ok((cmd.run(partial.client, partial.import_queue), partial.task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let partial = service::new_partial(&config)?;
                Ok((cmd.run(partial.client, config.database), partial.task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let partial = service::new_partial(&config)?;
                Ok((cmd.run(partial.client, config.chain_spec), partial.task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let partial = service::new_partial(&config)?;
                Ok((cmd.run(partial.client, partial.import_queue), partial.task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let partial = service::new_partial(&config)?;
                Ok((cmd.run(partial.client, partial.backend, None), partial.task_manager))
            })
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};

                let partial = service::new_partial(&config)?;
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => {
                        cmd.run::<ared_edge_runtime::Block, ()>(config)
                    }
                    BenchmarkCmd::Block(cmd) => cmd.run(partial.client),
                    BenchmarkCmd::Storage(cmd) => {
                        cmd.run(partial.client, partial.backend)
                    }
                    BenchmarkCmd::Overhead(_) => {
                        Err("Overhead benchmarking not supported".into())
                    }
                    BenchmarkCmd::Extrinsic(_) => {
                        Err("Extrinsic benchmarking not supported".into())
                    }
                    BenchmarkCmd::Machine(cmd) => {
                        cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
                    }
                }
            })
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<ared_edge_runtime::opaque::Block>(&config))
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config).map_err(sc_cli::Error::Service)
            })
        }
    }
}
