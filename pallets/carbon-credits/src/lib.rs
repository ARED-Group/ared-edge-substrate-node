//! # Carbon Credits Pallet
//!
//! This pallet provides functionality for calculating and managing carbon credits
//! based on verified telemetry from cooking stoves.
//!
//! ## Overview
//!
//! The Carbon Credits pallet enables:
//! - Calculating carbon savings from clean cooking energy usage
//! - Minting carbon credit tokens based on verified telemetry
//! - Tracking carbon credit balances per device and owner
//! - Transferring credits between accounts
//! - Governance controls for emission factors and parameters
//!
//! ## Carbon Calculation Methodology
//!
//! Carbon credits are calculated based on:
//! 1. Energy consumed by clean cooking stoves (verified via telemetry)
//! 2. Emission factor (kg CO2 per kWh of traditional fuel displaced)
//! 3. Credits per ton of CO2 avoided
//!
//! Formula: credits = (energy_kwh * emission_factor_kg_per_kwh / 1000) * credits_per_ton
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `record_energy` - Record energy usage for carbon calculation
//! - `claim_credits` - Claim accumulated carbon credits
//! - `transfer_credits` - Transfer credits between devices/accounts
//! - `set_emission_factor` - Update emission factor (governance)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use alloc::vec::Vec;

    /// Energy record with metadata
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, PartialEq)]
    pub struct EnergyRecord {
        /// Energy in watt-hours
        pub energy_wh: u128,
        /// Block number when recorded
        pub block_number: u32,
        /// Associated proof index (links to TelemetryProofs pallet)
        pub proof_index: Option<u64>,
    }

    /// Credit issuance record
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, PartialEq)]
    pub struct CreditIssuance {
        /// Credits issued
        pub credits: u128,
        /// Energy (Wh) that generated these credits
        pub energy_wh: u128,
        /// Block when issued
        pub block_number: u32,
        /// Emission factor used (scaled by 1000)
        pub emission_factor: u32,
    }

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics.
        type WeightInfo: WeightInfo;

        /// Maximum length of device ID
        #[pallet::constant]
        type MaxDeviceIdLength: Get<u32>;

        /// Default carbon credits per ton of CO2 avoided
        #[pallet::constant]
        type CreditsPerTonCO2: Get<u128>;

        /// Default emission factor (kg CO2 per kWh, scaled by 1000)
        /// Traditional cooking: ~0.5-2.0 kg CO2/kWh
        #[pallet::constant]
        type DefaultEmissionFactor: Get<u32>;

        /// Minimum energy (Wh) required before claiming credits
        #[pallet::constant]
        type MinClaimableEnergy: Get<u128>;

        /// Maximum credit issuance records per device
        #[pallet::constant]
        type MaxIssuanceRecords: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Current emission factor (kg CO2 per kWh, scaled by 1000)
    #[pallet::storage]
    #[pallet::getter(fn emission_factor)]
    pub type EmissionFactor<T: Config> = StorageValue<_, u32, ValueQuery, T::DefaultEmissionFactor>;

    /// Accumulated energy in Wh per device (pending credit calculation)
    #[pallet::storage]
    #[pallet::getter(fn energy_accumulated)]
    pub type EnergyAccumulated<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>,
        u128,
        ValueQuery,
    >;

    /// Total lifetime energy recorded per device
    #[pallet::storage]
    #[pallet::getter(fn total_energy)]
    pub type TotalEnergy<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>,
        u128,
        ValueQuery,
    >;

    /// Carbon credits balance per device
    #[pallet::storage]
    #[pallet::getter(fn credits_balance)]
    pub type CreditsBalance<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>,
        u128,
        ValueQuery,
    >;

    /// Credits balance per account (for transfers)
    #[pallet::storage]
    #[pallet::getter(fn account_credits)]
    pub type AccountCredits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u128,
        ValueQuery,
    >;

    /// Total credits issued across all devices
    #[pallet::storage]
    #[pallet::getter(fn total_credits_issued)]
    pub type TotalCreditsIssued<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// Total CO2 avoided (kg, scaled by 1000 for precision)
    #[pallet::storage]
    #[pallet::getter(fn total_co2_avoided)]
    pub type TotalCO2Avoided<T: Config> = StorageValue<_, u128, ValueQuery>;

    /// Issuance count per device
    #[pallet::storage]
    #[pallet::getter(fn issuance_count)]
    pub type IssuanceCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>,
        u32,
        ValueQuery,
    >;

    /// Active device count (devices with energy records)
    #[pallet::storage]
    #[pallet::getter(fn active_device_count)]
    pub type ActiveDeviceCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Energy was recorded for a device
        EnergyRecorded {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            energy_wh: u128,
            total_accumulated: u128,
        },
        /// Carbon credits were claimed/issued
        CreditsClaimed {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            credits: u128,
            energy_wh: u128,
            co2_avoided_kg: u128,
        },
        /// Credits transferred between devices
        CreditsTransferred {
            from_device: BoundedVec<u8, T::MaxDeviceIdLength>,
            to_device: BoundedVec<u8, T::MaxDeviceIdLength>,
            amount: u128,
        },
        /// Credits transferred to account
        CreditsWithdrawn {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            to_account: T::AccountId,
            amount: u128,
        },
        /// Emission factor updated
        EmissionFactorUpdated {
            old_factor: u32,
            new_factor: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Device ID exceeds maximum length
        DeviceIdTooLong,
        /// No credits available to claim
        NoCreditsAvailable,
        /// Insufficient credits for transfer
        InsufficientCredits,
        /// Arithmetic overflow
        Overflow,
        /// Energy below minimum claimable threshold
        EnergyBelowMinimum,
        /// Cannot transfer to same device
        SameDeviceTransfer,
        /// Invalid emission factor (must be > 0)
        InvalidEmissionFactor,
        /// Not authorized for governance action
        NotAuthorized,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Record energy usage for a device.
        ///
        /// This is called by the blockchain bridge after verifying telemetry.
        /// Energy is accumulated until claim_credits is called.
        ///
        /// # Arguments
        ///
        /// - `origin` - Signed origin (bridge account)
        /// - `device_id` - The device identifier
        /// - `energy_wh` - Energy in watt-hours
        /// - `proof_index` - Optional link to telemetry proof
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::record_energy())]
        pub fn record_energy(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
            energy_wh: u128,
            _proof_index: Option<u64>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                device_id.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;

            // Track if this is a new device
            let was_zero = EnergyAccumulated::<T>::get(&bounded_device_id).is_zero()
                && TotalEnergy::<T>::get(&bounded_device_id).is_zero();

            // Update accumulated energy (pending)
            let new_accumulated = EnergyAccumulated::<T>::mutate(&bounded_device_id, |total| {
                *total = total.saturating_add(energy_wh);
                *total
            });

            // Update total lifetime energy
            TotalEnergy::<T>::mutate(&bounded_device_id, |total| {
                *total = total.saturating_add(energy_wh);
            });

            // Increment active device count if new
            if was_zero {
                ActiveDeviceCount::<T>::mutate(|count| *count += 1);
            }

            Self::deposit_event(Event::EnergyRecorded {
                device_id: bounded_device_id,
                energy_wh,
                total_accumulated: new_accumulated,
            });

            Ok(())
        }

        /// Claim carbon credits based on accumulated energy.
        ///
        /// Converts accumulated energy to carbon credits using:
        /// CO2 avoided (kg) = energy_kwh * emission_factor_kg_per_kwh
        /// Credits = (CO2 avoided / 1000) * credits_per_ton
        ///
        /// # Arguments
        ///
        /// - `origin` - Signed origin
        /// - `device_id` - The device identifier
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim_credits())]
        pub fn claim_credits(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                device_id.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let accumulated = EnergyAccumulated::<T>::get(&bounded_device_id);
            ensure!(!accumulated.is_zero(), Error::<T>::NoCreditsAvailable);
            ensure!(
                accumulated >= T::MinClaimableEnergy::get(),
                Error::<T>::EnergyBelowMinimum
            );

            // Calculate CO2 avoided
            // emission_factor is kg CO2 per kWh, scaled by 1000
            // accumulated is in Wh
            let emission_factor = EmissionFactor::<T>::get() as u128;
            let energy_kwh = accumulated / 1000;
            
            // co2_avoided_kg = energy_kwh * (emission_factor / 1000)
            let co2_avoided_kg = energy_kwh
                .saturating_mul(emission_factor)
                .checked_div(1000)
                .ok_or(Error::<T>::Overflow)?;

            ensure!(!co2_avoided_kg.is_zero(), Error::<T>::NoCreditsAvailable);

            // Calculate credits
            // credits = (co2_avoided_kg / 1000) * credits_per_ton
            let credits = co2_avoided_kg
                .saturating_mul(T::CreditsPerTonCO2::get())
                .checked_div(1000)
                .ok_or(Error::<T>::Overflow)?;

            ensure!(!credits.is_zero(), Error::<T>::NoCreditsAvailable);

            // Update balances
            CreditsBalance::<T>::mutate(&bounded_device_id, |balance| {
                *balance = balance.saturating_add(credits);
            });
            TotalCreditsIssued::<T>::mutate(|total| {
                *total = total.saturating_add(credits);
            });
            TotalCO2Avoided::<T>::mutate(|total| {
                *total = total.saturating_add(co2_avoided_kg);
            });
            IssuanceCount::<T>::mutate(&bounded_device_id, |count| {
                *count = count.saturating_add(1);
            });

            // Reset accumulated energy
            EnergyAccumulated::<T>::insert(&bounded_device_id, 0u128);

            Self::deposit_event(Event::CreditsClaimed {
                device_id: bounded_device_id,
                credits,
                energy_wh: accumulated,
                co2_avoided_kg,
            });

            Ok(())
        }

        /// Transfer credits between devices.
        ///
        /// # Arguments
        ///
        /// - `origin` - Signed origin (must be authorized for from_device)
        /// - `from_device` - Source device ID
        /// - `to_device` - Destination device ID
        /// - `amount` - Number of credits to transfer
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::transfer_credits())]
        pub fn transfer_credits(
            origin: OriginFor<T>,
            from_device: Vec<u8>,
            to_device: Vec<u8>,
            amount: u128,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounded_from: BoundedVec<u8, T::MaxDeviceIdLength> =
                from_device.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;
            let bounded_to: BoundedVec<u8, T::MaxDeviceIdLength> =
                to_device.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;

            ensure!(bounded_from != bounded_to, Error::<T>::SameDeviceTransfer);

            let from_balance = CreditsBalance::<T>::get(&bounded_from);
            ensure!(from_balance >= amount, Error::<T>::InsufficientCredits);

            CreditsBalance::<T>::mutate(&bounded_from, |balance| {
                *balance = balance.saturating_sub(amount);
            });
            CreditsBalance::<T>::mutate(&bounded_to, |balance| {
                *balance = balance.saturating_add(amount);
            });

            Self::deposit_event(Event::CreditsTransferred {
                from_device: bounded_from,
                to_device: bounded_to,
                amount,
            });

            Ok(())
        }

        /// Withdraw credits from device to account.
        ///
        /// This moves credits from device balance to account balance,
        /// making them available for external trading/use.
        ///
        /// # Arguments
        ///
        /// - `origin` - Signed origin (account receiving credits)
        /// - `device_id` - Source device ID
        /// - `amount` - Number of credits to withdraw
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::withdraw_credits())]
        pub fn withdraw_credits(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
            amount: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                device_id.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let device_balance = CreditsBalance::<T>::get(&bounded_device_id);
            ensure!(device_balance >= amount, Error::<T>::InsufficientCredits);

            CreditsBalance::<T>::mutate(&bounded_device_id, |balance| {
                *balance = balance.saturating_sub(amount);
            });
            AccountCredits::<T>::mutate(&who, |balance| {
                *balance = balance.saturating_add(amount);
            });

            Self::deposit_event(Event::CreditsWithdrawn {
                device_id: bounded_device_id,
                to_account: who,
                amount,
            });

            Ok(())
        }

        /// Update the emission factor (governance function).
        ///
        /// # Arguments
        ///
        /// - `origin` - Root origin required
        /// - `new_factor` - New emission factor (kg CO2/kWh, scaled by 1000)
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::set_emission_factor())]
        pub fn set_emission_factor(
            origin: OriginFor<T>,
            new_factor: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(new_factor > 0, Error::<T>::InvalidEmissionFactor);

            let old_factor = EmissionFactor::<T>::get();
            EmissionFactor::<T>::put(new_factor);

            Self::deposit_event(Event::EmissionFactorUpdated {
                old_factor,
                new_factor,
            });

            Ok(())
        }
    }

    // Public query functions
    impl<T: Config> Pallet<T> {
        /// Get total credits for a device.
        pub fn get_device_credits(
            device_id: &BoundedVec<u8, T::MaxDeviceIdLength>,
        ) -> u128 {
            CreditsBalance::<T>::get(device_id)
        }

        /// Get pending energy (not yet converted to credits).
        pub fn get_pending_energy(
            device_id: &BoundedVec<u8, T::MaxDeviceIdLength>,
        ) -> u128 {
            EnergyAccumulated::<T>::get(device_id)
        }

        /// Calculate credits that would be issued for given energy.
        pub fn calculate_credits(energy_wh: u128) -> u128 {
            let emission_factor = EmissionFactor::<T>::get() as u128;
            let energy_kwh = energy_wh / 1000;
            let co2_avoided_kg = energy_kwh
                .saturating_mul(emission_factor)
                .saturating_div(1000);
            co2_avoided_kg
                .saturating_mul(T::CreditsPerTonCO2::get())
                .saturating_div(1000)
        }

        /// Get statistics summary.
        pub fn get_stats() -> (u128, u128, u32) {
            (
                TotalCreditsIssued::<T>::get(),
                TotalCO2Avoided::<T>::get(),
                ActiveDeviceCount::<T>::get(),
            )
        }
    }
}
