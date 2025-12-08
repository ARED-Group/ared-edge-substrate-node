//! ARED Edge Runtime
//!
//! The runtime for ARED Edge blockchain node, implementing
//! telemetry proofs, carbon calculations, and device management.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

// Provide panic handler for WASM no_std builds
#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Provide global allocator for WASM no_std builds
#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[global_allocator]
static ALLOCATOR: sp_core::alloc::WasmAllocator = sp_core::alloc::WasmAllocator;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    generic, impl_opaque_keys,
    traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, ExtrinsicInclusionMode, MultiSignature,
};
use alloc::borrow::Cow;
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

// Frame imports
use frame_support::{
    construct_runtime, derive_impl,
    weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
};
use pallet_grandpa::AuthorityList as GrandpaAuthorityList;
use pallet_transaction_payment::FungibleAdapter;
use sp_arithmetic::FixedU128;

/// Alias for account ID type
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Alias for signature type
pub type Signature = MultiSignature;

/// Balance type
pub type Balance = u128;

/// Block number type
pub type BlockNumber = u32;

/// Index type for transaction ordering
pub type Nonce = u32;

/// Hash type
pub type Hash = sp_core::H256;

/// Block header type
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// Signed block type
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type
pub type BlockId = generic::BlockId<Block>;

/// Unchecked extrinsic type
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<sp_runtime::MultiAddress<AccountId, ()>, RuntimeCall, Signature, SignedExtra>;

/// Executive type
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

/// Signed extras for transactions
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// Opaque types for light client
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

/// Runtime version
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("ared-edge"),
    impl_name: Cow::Borrowed("ared-edge-node"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

/// Maximum block weight
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
    WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
    u64::MAX,
);

/// Block execution time target (6 seconds)
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

/// Time constants
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// Native version for debugging
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
    sp_version::NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

// Frame system configuration
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
    type Block = Block;
    type BlockHashCount = frame_support::traits::ConstU32<256>;
    type Version = ();
    type AccountData = pallet_balances::AccountData<Balance>;
}

// Timestamp pallet configuration
impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = frame_support::traits::ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

// Aura consensus configuration
impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = frame_support::traits::ConstU32<32>;
    type AllowMultipleBlocksPerSlot = frame_support::traits::ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

// Grandpa finality configuration
impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = frame_support::traits::ConstU32<32>;
    type MaxNominators = frame_support::traits::ConstU32<0>;
    type MaxSetIdSessionEntries = ();
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

// Balances pallet configuration
impl pallet_balances::Config for Runtime {
    type MaxLocks = frame_support::traits::ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = frame_support::traits::ConstU128<500>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type DoneSlashHandler = ();
}

/// Constant fee multiplier for transaction payment
pub struct ConstantFeeMultiplier;
impl frame_support::traits::Get<FixedU128> for ConstantFeeMultiplier {
    fn get() -> FixedU128 {
        FixedU128::from_u32(1)
    }
}

// Transaction payment configuration
impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = frame_support::traits::ConstU8<5>;
    type WeightToFee = frame_support::weights::IdentityFee<Balance>;
    type LengthToFee = frame_support::weights::IdentityFee<Balance>;
    type FeeMultiplierUpdate = pallet_transaction_payment::ConstFeeMultiplier<ConstantFeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

// Sudo pallet configuration (for development)
impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// ARED Telemetry Proofs pallet configuration
impl pallet_telemetry_proofs::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_telemetry_proofs::weights::SubstrateWeight<Runtime>;
    /// Maximum device ID length (UUID = 36 chars, with buffer = 64)
    type MaxDeviceIdLength = frame_support::traits::ConstU32<64>;
    /// Maximum proof hash length (SHA-256 hex = 64 chars, with buffer = 128)
    type MaxProofLength = frame_support::traits::ConstU32<128>;
    /// Maximum proofs in a single batch submission
    type MaxBatchSize = frame_support::traits::ConstU32<100>;
    /// Maximum proof records per device (data retention limit)
    type MaxProofsPerDevice = frame_support::traits::ConstU32<10000>;
}

// ARED Carbon Credits pallet configuration
impl pallet_carbon_credits::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_carbon_credits::weights::SubstrateWeight<Runtime>;
    /// Maximum device ID length
    type MaxDeviceIdLength = frame_support::traits::ConstU32<64>;
    /// Credits per ton of CO2 avoided (1000 credits = 1 carbon credit token)
    type CreditsPerTonCO2 = frame_support::traits::ConstU128<1000>;
    /// Default emission factor: 1.5 kg CO2/kWh (scaled by 1000)
    /// Based on traditional biomass cooking displacement
    type DefaultEmissionFactor = frame_support::traits::ConstU32<1500>;
    /// Minimum energy (Wh) before claiming credits (1 kWh = 1000 Wh)
    type MinClaimableEnergy = frame_support::traits::ConstU128<1000>;
    /// Maximum issuance records per device
    type MaxIssuanceRecords = frame_support::traits::ConstU32<10000>;
}

// Construct the runtime
construct_runtime!(
    pub enum Runtime {
        // Core pallets
        System: frame_system,
        Timestamp: pallet_timestamp,

        // Consensus
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,

        // Monetary
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,

        // Governance (dev only)
        Sudo: pallet_sudo,

        // ARED Custom Pallets
        TelemetryProofs: pallet_telemetry_proofs,
        CarbonCredits: pallet_carbon_credits,
    }
);

// Implement runtime APIs
impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
        }

        fn authorities() -> Vec<AuraId> {
            pallet_aura::Authorities::<Runtime>::get().into_inner()
        }
    }

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            _key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            _authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            None
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }

        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }

        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }

        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, |_| None)
        }

        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
            vec![]
        }
    }
}
