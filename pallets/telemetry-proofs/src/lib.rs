//! # Telemetry Proofs Pallet
//!
//! This pallet provides functionality for storing and verifying telemetry proofs
//! from the ARED Edge IoT Platform.
//!
//! ## Overview
//!
//! The Telemetry Proofs pallet enables:
//! - Storing cryptographic commitments of telemetry data batches
//! - Linking telemetry to device identities
//! - Providing on-chain verification of data integrity
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `submit_proof` - Submit a new telemetry proof for a device
//! - `verify_proof` - Verify a telemetry proof exists

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Maximum length of device ID
        #[pallet::constant]
        type MaxDeviceIdLength: Get<u32>;

        /// Maximum length of proof hash
        #[pallet::constant]
        type MaxProofLength: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Telemetry proof storage
    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>, // device_id
        Blake2_128Concat,
        BlockNumberFor<T>, // block when submitted
        BoundedVec<u8, T::MaxProofLength>, // proof hash
        OptionQuery,
    >;

    /// Proof count per device
    #[pallet::storage]
    #[pallet::getter(fn proof_count)]
    pub type ProofCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>,
        u64,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A telemetry proof was submitted
        ProofSubmitted {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            proof_hash: BoundedVec<u8, T::MaxProofLength>,
            block_number: BlockNumberFor<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Device ID exceeds maximum length
        DeviceIdTooLong,
        /// Proof hash exceeds maximum length
        ProofTooLong,
        /// Proof already exists for this block
        ProofAlreadyExists,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a telemetry proof for a device.
        ///
        /// - `device_id`: The device identifier
        /// - `proof_hash`: The cryptographic hash of the telemetry batch
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn submit_proof(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
            proof_hash: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                device_id.try_into().map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let bounded_proof: BoundedVec<u8, T::MaxProofLength> =
                proof_hash.try_into().map_err(|_| Error::<T>::ProofTooLong)?;

            let current_block = <frame_system::Pallet<T>>::block_number();

            ensure!(
                !Proofs::<T>::contains_key(&bounded_device_id, current_block),
                Error::<T>::ProofAlreadyExists
            );

            Proofs::<T>::insert(&bounded_device_id, current_block, bounded_proof.clone());
            ProofCount::<T>::mutate(&bounded_device_id, |count| *count += 1);

            Self::deposit_event(Event::ProofSubmitted {
                device_id: bounded_device_id,
                proof_hash: bounded_proof,
                block_number: current_block,
            });

            Ok(())
        }
    }
}
