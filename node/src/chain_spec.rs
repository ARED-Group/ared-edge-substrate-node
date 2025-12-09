//! Chain specification for ARED Edge network.
//!
//! Defines genesis configuration for different network types:
//! - Development (single node for local testing)
//! - Local testnet (multi-node local network)
//! - Production (mainnet deployment)
//!
//! ## Account Structure
//!
//! The ARED Edge network uses a hierarchy of accounts:
//! - **Root/Sudo**: Administrative control (development only)
//! - **Bridge Account**: Submits telemetry proofs from the Ingest Service
//! - **Validator Accounts**: Block producers (Aura) and finalizers (Grandpa)
//! - **Treasury**: Collects transaction fees (future use)
//!
//! ## Genesis Configuration
//!
//! Initial state includes:
//! - Pre-funded accounts for operations
//! - Initial validator set
//! - Pallet configurations

use ared_edge_runtime::WASM_BINARY;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use serde_json::json;

/// Specialized `ChainSpec` for ARED Edge network.
pub type ChainSpec = sc_service::GenericChainSpec;

/// The type for account identifiers.
pub type AccountId = <<sp_runtime::MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Initial balance for pre-funded accounts (in smallest units).
const INITIAL_BALANCE: u128 = 1_000_000_000_000_000_000; // 1e18

/// Initial balance for bridge account.
const BRIDGE_BALANCE: u128 = 100_000_000_000_000_000; // 0.1e18

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    <TPublic::Pair as Pair>::Public: Into<AccountId>,
{
    get_from_seed::<TPublic>(seed).into()
}

/// Generate Aura authority key from seed.
pub fn get_aura_id_from_seed(seed: &str) -> AuraId {
    get_from_seed::<AuraId>(seed)
}

/// Generate Grandpa authority key from seed.
pub fn get_grandpa_id_from_seed(seed: &str) -> GrandpaId {
    get_from_seed::<GrandpaId>(seed)
}

/// Generate authority keys (Aura, Grandpa) from seed.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_aura_id_from_seed(s), get_grandpa_id_from_seed(s))
}

/// Development chain configuration (single node).
///
/// Characteristics:
/// - Single validator (Alice)
/// - Sudo enabled for administrative operations
/// - Fast block times for testing
/// - Pre-funded development accounts
pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name("ARED Edge Development")
    .with_id("ared_edge_dev")
    .with_chain_type(ChainType::Development)
    .with_protocol_id("ared-edge-dev")
    .with_genesis_config_patch(development_genesis_config())
    .with_properties(chain_properties())
    .build())
}

/// Local testnet configuration (multi-node).
///
/// Characteristics:
/// - Two validators (Alice, Bob)
/// - Sudo enabled for testing
/// - Pre-funded test accounts
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name("ARED Edge Local Testnet")
    .with_id("ared_edge_local")
    .with_chain_type(ChainType::Local)
    .with_protocol_id("ared-edge-local")
    .with_genesis_config_patch(local_testnet_genesis_config())
    .with_properties(chain_properties())
    .build())
}

/// Production chain configuration.
///
/// Characteristics:
/// - Multiple validators for security
/// - Sudo disabled (governance-based administration)
/// - Conservative resource limits
/// - Production-ready parameters
pub fn production_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Production wasm not available".to_string())?,
        None,
    )
    .with_name("ARED Edge Mainnet")
    .with_id("ared_edge_mainnet")
    .with_chain_type(ChainType::Live)
    .with_protocol_id("ared-edge")
    .with_genesis_config_patch(production_genesis_config())
    .with_properties(chain_properties())
    .build())
}

/// Chain properties for wallet and explorer integration.
fn chain_properties() -> serde_json::Map<String, serde_json::Value> {
    let mut properties = serde_json::Map::new();
    properties.insert("tokenSymbol".into(), "ARED".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 42.into());
    properties
}

/// Genesis configuration for development network.
fn development_genesis_config() -> serde_json::Value {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let bridge = get_account_id_from_seed::<sr25519::Public>("Bridge");
    
    let (alice_aura, alice_grandpa) = authority_keys_from_seed("Alice");

    json!({
        "balances": {
            "balances": [
                [alice.to_string(), INITIAL_BALANCE],
                [bob.to_string(), INITIAL_BALANCE / 10],
                [bridge.to_string(), BRIDGE_BALANCE]
            ]
        },
        "aura": {
            "authorities": [alice_aura.to_string()]
        },
        "grandpa": {
            "authorities": [[alice_grandpa.to_string(), 1]]
        },
        "sudo": {
            "key": alice.to_string()
        }
    })
}

/// Genesis configuration for local testnet.
fn local_testnet_genesis_config() -> serde_json::Value {
    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
    let bob = get_account_id_from_seed::<sr25519::Public>("Bob");
    let charlie = get_account_id_from_seed::<sr25519::Public>("Charlie");
    let bridge = get_account_id_from_seed::<sr25519::Public>("Bridge");
    
    let (alice_aura, alice_grandpa) = authority_keys_from_seed("Alice");
    let (bob_aura, bob_grandpa) = authority_keys_from_seed("Bob");

    json!({
        "balances": {
            "balances": [
                [alice.to_string(), INITIAL_BALANCE],
                [bob.to_string(), INITIAL_BALANCE],
                [charlie.to_string(), INITIAL_BALANCE / 10],
                [bridge.to_string(), BRIDGE_BALANCE]
            ]
        },
        "aura": {
            "authorities": [
                alice_aura.to_string(),
                bob_aura.to_string()
            ]
        },
        "grandpa": {
            "authorities": [
                [alice_grandpa.to_string(), 1],
                [bob_grandpa.to_string(), 1]
            ]
        },
        "sudo": {
            "key": alice.to_string()
        }
    })
}

/// Genesis configuration for production network.
///
/// Note: In production, actual validator keys should be generated
/// securely and not from seeds. This is a template.
fn production_genesis_config() -> serde_json::Value {
    // Production accounts would be configured from environment or secure key management
    // These are placeholder values that MUST be replaced before mainnet launch
    let root = get_account_id_from_seed::<sr25519::Public>("Root");
    let bridge = get_account_id_from_seed::<sr25519::Public>("Bridge");
    let validator1 = get_account_id_from_seed::<sr25519::Public>("Validator1");
    let validator2 = get_account_id_from_seed::<sr25519::Public>("Validator2");
    let validator3 = get_account_id_from_seed::<sr25519::Public>("Validator3");

    let (v1_aura, v1_grandpa) = authority_keys_from_seed("Validator1");
    let (v2_aura, v2_grandpa) = authority_keys_from_seed("Validator2");
    let (v3_aura, v3_grandpa) = authority_keys_from_seed("Validator3");

    json!({
        "balances": {
            "balances": [
                [root.to_string(), INITIAL_BALANCE],
                [bridge.to_string(), BRIDGE_BALANCE],
                [validator1.to_string(), BRIDGE_BALANCE],
                [validator2.to_string(), BRIDGE_BALANCE],
                [validator3.to_string(), BRIDGE_BALANCE]
            ]
        },
        "aura": {
            "authorities": [
                v1_aura.to_string(),
                v2_aura.to_string(),
                v3_aura.to_string()
            ]
        },
        "grandpa": {
            "authorities": [
                [v1_grandpa.to_string(), 1],
                [v2_grandpa.to_string(), 1],
                [v3_grandpa.to_string(), 1]
            ]
        }
        // Note: No sudo in production - governance-based administration only
    })
}
