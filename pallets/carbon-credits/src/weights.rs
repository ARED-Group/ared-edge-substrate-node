//! Weight calculations for the Carbon Credits pallet.
//!
//! These weights ensure proper transaction fee calculation
//! and prevent denial-of-service attacks.

use frame_support::{traits::Get, weights::Weight};

/// Weight functions for the pallet.
pub trait WeightInfo {
    /// Weight for recording energy usage.
    fn record_energy() -> Weight;

    /// Weight for claiming carbon credits.
    fn claim_credits() -> Weight;

    /// Weight for transferring credits between devices.
    fn transfer_credits() -> Weight;

    /// Weight for withdrawing credits to account.
    fn withdraw_credits() -> Weight;

    /// Weight for setting emission factor.
    fn set_emission_factor() -> Weight;
}

/// Default weight implementation.
///
/// Placeholder weights that should be replaced with benchmark results.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Weight for recording energy.
    ///
    /// Operations:
    /// - Read accumulated energy
    /// - Read total energy
    /// - Check if new device
    /// - Write accumulated energy
    /// - Write total energy
    /// - Potentially increment device count
    /// - Emit event
    fn record_energy() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    /// Weight for claiming credits.
    ///
    /// Operations:
    /// - Read accumulated energy
    /// - Read emission factor
    /// - Calculate CO2 and credits
    /// - Update credits balance
    /// - Update total credits issued
    /// - Update total CO2 avoided
    /// - Update issuance count
    /// - Reset accumulated energy
    /// - Emit event
    fn claim_credits() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    /// Weight for transferring credits.
    ///
    /// Operations:
    /// - Read from balance
    /// - Write from balance
    /// - Write to balance
    /// - Emit event
    fn transfer_credits() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Weight for withdrawing credits to account.
    ///
    /// Operations:
    /// - Read device balance
    /// - Write device balance
    /// - Write account balance
    /// - Emit event
    fn withdraw_credits() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    /// Weight for setting emission factor.
    ///
    /// Operations:
    /// - Read current factor
    /// - Write new factor
    /// - Emit event
    fn set_emission_factor() -> Weight {
        Weight::from_parts(15_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// Unit implementation for testing.
impl WeightInfo for () {
    fn record_energy() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn claim_credits() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn transfer_credits() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn withdraw_credits() -> Weight {
        Weight::from_parts(10_000, 0)
    }

    fn set_emission_factor() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
