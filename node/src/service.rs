//! ARED Edge Node Service
//!
//! Minimal service layer for the ARED Edge Substrate node.
//! Uses polkadot-stable2503 APIs.

use std::sync::Arc;
use sc_executor::WasmExecutor;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use ared_edge_runtime::{opaque::Block, RuntimeApi};

pub type FullClient = sc_service::TFullClient<Block, RuntimeApi, WasmExecutor<sp_io::SubstrateHostFunctions>>;
pub type FullBackend = sc_service::TFullBackend<Block>;

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
    
    let (client, backend, keystore_container, mut task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            &config,
            telemetry.as_ref().map(|(_, t)| t.handle()),
            executor,
        )?;
    let client = Arc::new(client);

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
