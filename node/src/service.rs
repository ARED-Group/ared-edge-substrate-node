//! ARED Edge Node Service
//!
//! Full service implementation with Aura consensus and Grandpa finality.

use sc_executor::WasmExecutor;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use ared_edge_runtime::{opaque::Block, RuntimeApi};

/// Full client type alias.
pub type FullClient = sc_service::TFullClient<
    Block,
    RuntimeApi,
    WasmExecutor<sp_io::SubstrateHostFunctions>,
>;

/// Full backend type alias.
pub type FullBackend = sc_service::TFullBackend<Block>;

/// Select chain type alias.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Grandpa justification period for block finality.
#[allow(dead_code)]
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Service partial components type alias.
#[allow(dead_code)]
pub type Service = sc_service::PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::FullPool<Block, FullClient>,
    (
        sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
        sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
        Option<Telemetry>,
    ),
>;

/// Start a development node.
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    let telemetry = config.telemetry_endpoints.clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);
    
    let (_client, _backend, _keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            &config,
            telemetry.as_ref().map(|(_, t)| t.handle()),
            executor,
        )?;

    let _telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let role = config.role;
    
    if role.is_authority() {
        log::info!("Node is running as authority");
    }

    Ok(task_manager)
}
