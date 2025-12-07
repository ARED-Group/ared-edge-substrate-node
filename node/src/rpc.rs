//! RPC interface for ARED Edge node.
//!
//! This module provides JSON-RPC endpoints for external clients
//! to interact with the blockchain.

use std::sync::Arc;

use ared_edge_runtime::{opaque::Block, AccountId, Balance, Nonce};
use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;

pub use sc_rpc_api::DenyUnsafe;

/// Full client dependencies for RPC.
pub struct FullDeps<C, P> {
    /// The client instance.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls.
    pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: sp_api::ProvideRuntimeApi<Block>,
    C: sc_client_api::HeaderBackend<Block> + sc_client_api::AuxStore,
    C: sc_client_api::BlockchainEvents<Block>,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: sp_block_builder::BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client, pool, deny_unsafe } = deps;

    // System RPC (account nonce, etc.)
    module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;

    // Transaction payment RPC
    module.merge(TransactionPayment::new(client).into_rpc())?;

    Ok(module)
}
