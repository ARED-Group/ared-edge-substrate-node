//! # Carbon Credits Pallet
//!
//! This pallet provides functionality for calculating and managing carbon credits
//! based on verified telemetry from cooking stoves.
//!
//! ## Overview
//!
//! The Carbon Credits pallet enables:
//! - Calculating carbon savings from energy usage data
//! - Minting carbon credit tokens
//! - Tracking carbon credit balances per device/owner
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `record_energy` - Record energy usage for carbon calculation
//! - `claim_credits` - Claim accumulated carbon credits

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use sp_std::vec::Vec;

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Maximum length of device ID
        #[pallet::constant]
        type MaxDeviceIdLength: Get<u32>;

        /// Carbon credits per kWh of energy saved
        #[pallet::constant]
        type CreditsPerKwh: Get<u128>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Accumulated energy in Wh per device
    #[pallet::storage]
    #[pallet::getter(fn energy_accumulated)]
    pub type EnergyAccumulated<T: Config> = StorageMap<
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

    /// Total credits issued
    #[pallet::storage]
    #[pallet::getter(fn total_credits_issued)]
    pub type TotalCreditsIssued<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Energy was recorded for a device
        EnergyRecorded {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            energy_wh: u128,
            total_accumulated: u128,
        },
        /// Carbon credits were claimed
        CreditsClaimed {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            credits: u128,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Device ID exceeds maximum length
        DeviceIdTooLong,
        /// No credits available to claim
        NoCreditsAvailable,
        /// Arithmetic overflow
        Overflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Record energy usage for a device.
        ///
        /// - `device_id`: The device identifier
        /// - `energy_wh`: Energy in watt-hours
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn record_energy(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
            energy_wh: u128,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                device_id.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let new_total = EnergyAccumulated::<T>::mutate(&bounded_device_id, |total| {
                *total = total.saturating_add(energy_wh);
                *total
            });

            Self::deposit_event(Event::EnergyRecorded {
                device_id: bounded_device_id,
                energy_wh,
                total_accumulated: new_total,
            });

            Ok(())
        }

        /// Claim carbon credits based on accumulated energy.
        ///
        /// - `device_id`: The device identifier
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn claim_credits(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                device_id.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let accumulated = EnergyAccumulated::<T>::get(&bounded_device_id);
            ensure!(!accumulated.is_zero(), Error::<T>::NoCreditsAvailable);

            // Calculate credits: (Wh / 1000) * CreditsPerKwh
            let kwh = accumulated / 1000;
            let credits = kwh.saturating_mul(T::CreditsPerKwh::get());

            ensure!(!credits.is_zero(), Error::<T>::NoCreditsAvailable);

            // Update balances
            CreditsBalance::<T>::mutate(&bounded_device_id, |balance| {
                *balance = balance.saturating_add(credits);
            });
            TotalCreditsIssued::<T>::mutate(|total| {
                *total = total.saturating_add(credits);
            });

            // Reset accumulated energy
            EnergyAccumulated::<T>::insert(&bounded_device_id, 0u128);

            Self::deposit_event(Event::CreditsClaimed {
                device_id: bounded_device_id,
                credits,
            });

            Ok(())
        }
    }
}
