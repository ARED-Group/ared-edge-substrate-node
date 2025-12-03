//! Weight calculations for the Telemetry Proofs pallet.
//!
//! These weights are used to ensure proper transaction fee calculation
//! and prevent denial-of-service attacks through expensive operations.

use frame_support::weights::Weight;

/// Weight functions for the pallet.
pub trait WeightInfo {
    /// Weight for submitting a single proof.
    fn submit_proof() -> Weight;
    
    /// Weight for submitting a batch of proofs.
    fn submit_batch_proofs(n: u32) -> Weight;
    
    /// Weight for verifying a proof exists.
    fn verify_proof() -> Weight;
}

/// Default weight implementation.
///
/// These are placeholder weights that should be replaced with
/// actual benchmark results for production use.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Weight for submitting a single proof.
    ///
    /// Includes:
    /// - Reading proof count for device
    /// - Checking for duplicate at block
    /// - Writing proof metadata
    /// - Writing to proofs by block index
    /// - Updating proof count
    /// - Updating total proofs
    /// - Updating latest proof block
    /// - Emitting event
    fn submit_proof() -> Weight {
        // Base weight: ~50_000 + DB reads (2) + DB writes (5)
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    
    /// Weight for submitting a batch of proofs.
    ///
    /// Linear scaling with number of proofs.
    fn submit_batch_proofs(n: u32) -> Weight {
        // Base weight + per-proof weight
        Weight::from_parts(20_000_000, 0)
            .saturating_add(Weight::from_parts(40_000_000 * n as u64, 0))
            .saturating_add(T::DbWeight::get().reads(2 * n as u64))
            .saturating_add(T::DbWeight::get().writes(5 * n as u64))
    }
    
    /// Weight for verifying a proof exists.
    ///
    /// Includes:
    /// - Reading proof count
    /// - Potentially iterating through all proofs (worst case)
    /// - Emitting event
    fn verify_proof() -> Weight {
        // Worst case: iterate through MaxProofsPerDevice proofs
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1001)) // count + max proofs
    }
}

/// Unit implementation for testing.
impl WeightInfo for () {
    fn submit_proof() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn submit_batch_proofs(_n: u32) -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn verify_proof() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
