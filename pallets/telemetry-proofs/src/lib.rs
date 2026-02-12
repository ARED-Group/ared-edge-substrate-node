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
//! - Batch submission for efficient proof aggregation
//! - Query proofs by device and time range
//!
//! ## Integration
//!
//! The blockchain bridge service submits proofs after validating telemetry:
//! 1. Bridge receives Postgres NOTIFY for new telemetry
//! 2. Bridge aggregates telemetry into batches
//! 3. Bridge computes batch hash and submits proof
//! 4. Proof is stored on-chain with device association
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `submit_proof` - Submit a new telemetry proof for a device
//! - `submit_batch_proofs` - Submit multiple proofs in a single transaction
//! - `verify_proof` - Verify a telemetry proof exists on-chain

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
    use alloc::vec::Vec;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    };

    /// Proof metadata stored alongside the hash
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Debug, PartialEq)]
    #[scale_info(skip_type_params(T))]
    pub struct ProofMetadata<T: Config> {
        /// The proof hash (SHA-256 of telemetry batch)
        pub proof_hash: BoundedVec<u8, T::MaxProofLength>,
        /// Block number when proof was submitted
        pub block_number: BlockNumberFor<T>,
        /// Timestamp of submission (from pallet_timestamp if available)
        pub timestamp: u64,
        /// Number of telemetry records in this batch
        pub record_count: u32,
        /// Start time of the telemetry window (UNIX timestamp)
        pub window_start: u64,
        /// End time of the telemetry window (UNIX timestamp)
        pub window_end: u64,
    }

    /// The pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Maximum length of device ID (typically UUID = 36 bytes)
        #[pallet::constant]
        type MaxDeviceIdLength: Get<u32>;

        /// Maximum length of proof hash (SHA-256 = 32 bytes, hex = 64 bytes)
        #[pallet::constant]
        type MaxProofLength: Get<u32>;

        /// Maximum number of proofs in a batch submission
        #[pallet::constant]
        type MaxBatchSize: Get<u32>;

        /// Maximum number of proofs stored per device
        #[pallet::constant]
        type MaxProofsPerDevice: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Telemetry proofs indexed by device and proof index
    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>, // device_id
        Blake2_128Concat,
        u64, // proof index
        ProofMetadata<T>,
        OptionQuery,
    >;

    /// Proofs indexed by block number for time-range queries
    #[pallet::storage]
    #[pallet::getter(fn proofs_by_block)]
    pub type ProofsByBlock<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>, // block number
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>, // device_id
        BoundedVec<u8, T::MaxProofLength>,    // proof hash
        OptionQuery,
    >;

    /// Proof count per device (also serves as next proof index)
    #[pallet::storage]
    #[pallet::getter(fn proof_count)]
    pub type ProofCount<T: Config> =
        StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxDeviceIdLength>, u64, ValueQuery>;

    /// Total proofs submitted across all devices
    #[pallet::storage]
    #[pallet::getter(fn total_proofs)]
    pub type TotalProofs<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Latest proof block per device for quick lookup
    #[pallet::storage]
    #[pallet::getter(fn latest_proof_block)]
    pub type LatestProofBlock<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxDeviceIdLength>,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A telemetry proof was submitted
        ProofSubmitted {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            proof_hash: BoundedVec<u8, T::MaxProofLength>,
            block_number: BlockNumberFor<T>,
            proof_index: u64,
        },
        /// A batch of proofs was submitted
        BatchProofsSubmitted {
            submitter: T::AccountId,
            proof_count: u32,
            block_number: BlockNumberFor<T>,
        },
        /// A batch of unsigned proofs was submitted
        UnsignedBatchProofsSubmitted {
            proof_count: u32,
            block_number: BlockNumberFor<T>,
        },
        /// A proof was verified
        ProofVerified {
            device_id: BoundedVec<u8, T::MaxDeviceIdLength>,
            proof_hash: BoundedVec<u8, T::MaxProofLength>,
            exists: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Device ID exceeds maximum length
        DeviceIdTooLong,
        /// Proof hash exceeds maximum length
        ProofTooLong,
        /// Proof already exists for this device at this block
        ProofAlreadyExists,
        /// Batch size exceeds maximum allowed
        BatchTooLarge,
        /// Empty batch submitted
        EmptyBatch,
        /// Maximum proofs per device exceeded
        MaxProofsExceeded,
        /// Proof not found
        ProofNotFound,
        /// Invalid time window (start >= end)
        InvalidTimeWindow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a telemetry proof for a device.
        ///
        /// This extrinsic stores a cryptographic commitment of a telemetry batch.
        /// The proof_hash is typically a SHA-256 hash of the aggregated telemetry.
        ///
        /// # Arguments
        ///
        /// - `origin` - The transaction origin (must be signed by bridge account)
        /// - `device_id` - The device identifier (UUID format)
        /// - `proof_hash` - The cryptographic hash of the telemetry batch
        /// - `record_count` - Number of telemetry records in this batch
        /// - `window_start` - Start timestamp of the telemetry window
        /// - `window_end` - End timestamp of the telemetry window
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::submit_proof())]
        pub fn submit_proof(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
            proof_hash: Vec<u8>,
            record_count: u32,
            window_start: u64,
            window_end: u64,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Validate time window
            ensure!(window_start < window_end, Error::<T>::InvalidTimeWindow);

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> = device_id
                .try_into()
                .map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let bounded_proof: BoundedVec<u8, T::MaxProofLength> = proof_hash
                .try_into()
                .map_err(|_| Error::<T>::ProofTooLong)?;

            let current_block = <frame_system::Pallet<T>>::block_number();

            // Check max proofs per device
            let current_count = ProofCount::<T>::get(&bounded_device_id);
            ensure!(
                current_count < T::MaxProofsPerDevice::get() as u64,
                Error::<T>::MaxProofsExceeded
            );

            // Check for duplicate at same block
            ensure!(
                !ProofsByBlock::<T>::contains_key(current_block, &bounded_device_id),
                Error::<T>::ProofAlreadyExists
            );

            // Create proof metadata
            let metadata = ProofMetadata::<T> {
                proof_hash: bounded_proof.clone(),
                block_number: current_block,
                timestamp: Self::current_timestamp(),
                record_count,
                window_start,
                window_end,
            };

            // Store proof with index
            let proof_index = current_count;
            Proofs::<T>::insert(&bounded_device_id, proof_index, metadata);
            ProofsByBlock::<T>::insert(current_block, &bounded_device_id, bounded_proof.clone());
            ProofCount::<T>::mutate(&bounded_device_id, |count| *count += 1);
            TotalProofs::<T>::mutate(|total| *total += 1);
            LatestProofBlock::<T>::insert(&bounded_device_id, current_block);

            Self::deposit_event(Event::ProofSubmitted {
                device_id: bounded_device_id,
                proof_hash: bounded_proof,
                block_number: current_block,
                proof_index,
            });

            Ok(())
        }

        /// Submit multiple proofs in a single transaction.
        ///
        /// This is more efficient than multiple individual submissions as it
        /// amortizes the transaction overhead across all proofs.
        ///
        /// # Arguments
        ///
        /// - `origin` - The transaction origin (must be signed by bridge account)
        /// - `proofs` - Vector of (device_id, proof_hash, record_count, window_start, window_end)
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::submit_batch_proofs(proofs.len() as u32))]
        pub fn submit_batch_proofs(
            origin: OriginFor<T>,
            proofs: Vec<(Vec<u8>, Vec<u8>, u32, u64, u64)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let batch_len = proofs.len() as u32;
            ensure!(batch_len > 0, Error::<T>::EmptyBatch);
            ensure!(
                batch_len <= T::MaxBatchSize::get(),
                Error::<T>::BatchTooLarge
            );

            let current_block = <frame_system::Pallet<T>>::block_number();

            for (device_id, proof_hash, record_count, window_start, window_end) in proofs {
                // Validate time window
                if window_start >= window_end {
                    continue; // Skip invalid entries rather than fail entire batch
                }

                let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                    match device_id.try_into() {
                        Ok(id) => id,
                        Err(_) => continue, // Skip invalid device IDs
                    };

                let bounded_proof: BoundedVec<u8, T::MaxProofLength> = match proof_hash.try_into() {
                    Ok(hash) => hash,
                    Err(_) => continue, // Skip invalid proofs
                };

                // Check limits and duplicates
                let current_count = ProofCount::<T>::get(&bounded_device_id);
                if current_count >= T::MaxProofsPerDevice::get() as u64 {
                    continue;
                }
                if ProofsByBlock::<T>::contains_key(current_block, &bounded_device_id) {
                    continue;
                }

                // Create and store proof
                let metadata = ProofMetadata::<T> {
                    proof_hash: bounded_proof.clone(),
                    block_number: current_block,
                    timestamp: Self::current_timestamp(),
                    record_count,
                    window_start,
                    window_end,
                };

                let proof_index = current_count;
                Proofs::<T>::insert(&bounded_device_id, proof_index, metadata);
                ProofsByBlock::<T>::insert(
                    current_block,
                    &bounded_device_id,
                    bounded_proof.clone(),
                );
                ProofCount::<T>::mutate(&bounded_device_id, |count| *count += 1);
                TotalProofs::<T>::mutate(|total| *total += 1);
                LatestProofBlock::<T>::insert(&bounded_device_id, current_block);

                Self::deposit_event(Event::ProofSubmitted {
                    device_id: bounded_device_id,
                    proof_hash: bounded_proof,
                    block_number: current_block,
                    proof_index,
                });
            }

            Self::deposit_event(Event::BatchProofsSubmitted {
                submitter: who,
                proof_count: batch_len,
                block_number: current_block,
            });

            Ok(())
        }

        /// Verify that a proof exists for a device.
        ///
        /// This is primarily for debugging and verification purposes.
        /// Returns an event indicating whether the proof exists.
        ///
        /// # Arguments
        ///
        /// - `origin` - The transaction origin (can be any signed account)
        /// - `device_id` - The device identifier
        /// - `proof_hash` - The proof hash to verify
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::verify_proof())]
        pub fn verify_proof(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
            proof_hash: Vec<u8>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> = device_id
                .try_into()
                .map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let bounded_proof: BoundedVec<u8, T::MaxProofLength> = proof_hash
                .try_into()
                .map_err(|_| Error::<T>::ProofTooLong)?;

            // Search for the proof in device's proof history
            let proof_count = ProofCount::<T>::get(&bounded_device_id);
            let mut exists = false;

            for i in 0..proof_count {
                if let Some(metadata) = Proofs::<T>::get(&bounded_device_id, i) {
                    if metadata.proof_hash == bounded_proof {
                        exists = true;
                        break;
                    }
                }
            }

            Self::deposit_event(Event::ProofVerified {
                device_id: bounded_device_id,
                proof_hash: bounded_proof,
                exists,
            });

            Ok(())
        }

        /// Submit a telemetry proof without requiring a signed transaction.
        ///
        /// This extrinsic is validated via ValidateUnsigned and is intended for
        /// use by the blockchain bridge service where authentication happens at
        /// the MQTT/API layer rather than the blockchain layer.
        ///
        /// # Arguments
        ///
        /// - `origin` - Must be none (unsigned)
        /// - `device_id` - The device identifier (UUID format)
        /// - `proof_hash` - The cryptographic hash of the telemetry batch
        /// - `record_count` - Number of telemetry records in this batch
        /// - `window_start` - Start timestamp of the telemetry window
        /// - `window_end` - End timestamp of the telemetry window
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::submit_proof())]
        pub fn submit_proof_unsigned(
            origin: OriginFor<T>,
            device_id: Vec<u8>,
            proof_hash: Vec<u8>,
            record_count: u32,
            window_start: u64,
            window_end: u64,
        ) -> DispatchResult {
            ensure_none(origin)?;

            // Validate time window
            ensure!(window_start < window_end, Error::<T>::InvalidTimeWindow);

            let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> = device_id
                .try_into()
                .map_err(|_| Error::<T>::DeviceIdTooLong)?;

            let bounded_proof: BoundedVec<u8, T::MaxProofLength> = proof_hash
                .try_into()
                .map_err(|_| Error::<T>::ProofTooLong)?;

            let current_block = <frame_system::Pallet<T>>::block_number();

            // Check max proofs per device
            let current_count = ProofCount::<T>::get(&bounded_device_id);
            ensure!(
                current_count < T::MaxProofsPerDevice::get() as u64,
                Error::<T>::MaxProofsExceeded
            );

            // Check for duplicate at same block
            ensure!(
                !ProofsByBlock::<T>::contains_key(current_block, &bounded_device_id),
                Error::<T>::ProofAlreadyExists
            );

            // Create proof metadata
            let metadata = ProofMetadata::<T> {
                proof_hash: bounded_proof.clone(),
                block_number: current_block,
                timestamp: Self::current_timestamp(),
                record_count,
                window_start,
                window_end,
            };

            // Store proof with index
            let proof_index = current_count;
            Proofs::<T>::insert(&bounded_device_id, proof_index, metadata);
            ProofsByBlock::<T>::insert(current_block, &bounded_device_id, bounded_proof.clone());
            ProofCount::<T>::mutate(&bounded_device_id, |count| *count += 1);
            TotalProofs::<T>::mutate(|total| *total += 1);
            LatestProofBlock::<T>::insert(&bounded_device_id, current_block);

            Self::deposit_event(Event::ProofSubmitted {
                device_id: bounded_device_id,
                proof_hash: bounded_proof,
                block_number: current_block,
                proof_index,
            });

            Ok(())
        }

        /// Submit multiple proofs in a single unsigned transaction.
        ///
        /// This is more efficient than multiple individual submissions.
        /// Validated via ValidateUnsigned for bridge service use.
        ///
        /// # Arguments
        ///
        /// - `origin` - Must be none (unsigned)
        /// - `proofs` - Vector of (device_id, proof_hash, record_count, window_start, window_end)
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::submit_batch_proofs(proofs.len() as u32))]
        pub fn submit_batch_proofs_unsigned(
            origin: OriginFor<T>,
            proofs: Vec<(Vec<u8>, Vec<u8>, u32, u64, u64)>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            let batch_len = proofs.len() as u32;
            ensure!(batch_len > 0, Error::<T>::EmptyBatch);
            ensure!(
                batch_len <= T::MaxBatchSize::get(),
                Error::<T>::BatchTooLarge
            );

            let current_block = <frame_system::Pallet<T>>::block_number();
            let mut successful_count = 0u32;

            for (device_id, proof_hash, record_count, window_start, window_end) in proofs {
                if window_start >= window_end {
                    continue;
                }

                let bounded_device_id: BoundedVec<u8, T::MaxDeviceIdLength> =
                    match device_id.try_into() {
                        Ok(id) => id,
                        Err(_) => continue,
                    };

                let bounded_proof: BoundedVec<u8, T::MaxProofLength> = match proof_hash.try_into() {
                    Ok(hash) => hash,
                    Err(_) => continue,
                };

                let current_count = ProofCount::<T>::get(&bounded_device_id);
                if current_count >= T::MaxProofsPerDevice::get() as u64 {
                    continue;
                }
                if ProofsByBlock::<T>::contains_key(current_block, &bounded_device_id) {
                    continue;
                }

                let metadata = ProofMetadata::<T> {
                    proof_hash: bounded_proof.clone(),
                    block_number: current_block,
                    timestamp: Self::current_timestamp(),
                    record_count,
                    window_start,
                    window_end,
                };

                let proof_index = current_count;
                Proofs::<T>::insert(&bounded_device_id, proof_index, metadata);
                ProofsByBlock::<T>::insert(
                    current_block,
                    &bounded_device_id,
                    bounded_proof.clone(),
                );
                ProofCount::<T>::mutate(&bounded_device_id, |count| *count += 1);
                TotalProofs::<T>::mutate(|total| *total += 1);
                LatestProofBlock::<T>::insert(&bounded_device_id, current_block);

                Self::deposit_event(Event::ProofSubmitted {
                    device_id: bounded_device_id,
                    proof_hash: bounded_proof,
                    block_number: current_block,
                    proof_index,
                });

                successful_count += 1;
            }

            Self::deposit_event(Event::UnsignedBatchProofsSubmitted {
                proof_count: successful_count,
                block_number: current_block,
            });

            Ok(())
        }
    }

    // Public query functions for runtime APIs
    impl<T: Config> Pallet<T> {
        /// Read the current on-chain timestamp from pallet_timestamp.
        fn current_timestamp() -> u64 {
            let moment = <pallet_timestamp::Pallet<T>>::get();
            moment.try_into().unwrap_or(0)
        }

        /// Get proof metadata by device and index.
        pub fn get_proof(
            device_id: &BoundedVec<u8, T::MaxDeviceIdLength>,
            index: u64,
        ) -> Option<ProofMetadata<T>> {
            Proofs::<T>::get(device_id, index)
        }

        /// Get all proofs for a device.
        pub fn get_device_proofs(
            device_id: &BoundedVec<u8, T::MaxDeviceIdLength>,
        ) -> Vec<ProofMetadata<T>> {
            let count = ProofCount::<T>::get(device_id);
            (0..count)
                .filter_map(|i| Proofs::<T>::get(device_id, i))
                .collect()
        }

        /// Check if a specific proof hash exists for a device.
        pub fn proof_exists(
            device_id: &BoundedVec<u8, T::MaxDeviceIdLength>,
            proof_hash: &BoundedVec<u8, T::MaxProofLength>,
        ) -> bool {
            let count = ProofCount::<T>::get(device_id);
            for i in 0..count {
                if let Some(metadata) = Proofs::<T>::get(device_id, i) {
                    if &metadata.proof_hash == proof_hash {
                        return true;
                    }
                }
            }
            false
        }

        /// Get proofs within a time window for a device.
        pub fn get_proofs_in_window(
            device_id: &BoundedVec<u8, T::MaxDeviceIdLength>,
            start_time: u64,
            end_time: u64,
        ) -> Vec<ProofMetadata<T>> {
            let count = ProofCount::<T>::get(device_id);
            (0..count)
                .filter_map(|i| Proofs::<T>::get(device_id, i))
                .filter(|m| m.window_start >= start_time && m.window_end <= end_time)
                .collect()
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            // Only accept unsigned extrinsics from local sources (co-located bridge).
            // InBlock is also accepted for re-validation during block import.
            if !matches!(source, TransactionSource::Local | TransactionSource::InBlock) {
                return InvalidTransaction::BadSigner.into();
            }

            match call {
                Call::submit_proof_unsigned {
                    device_id,
                    proof_hash,
                    window_start,
                    window_end,
                    ..
                } => {
                    if device_id.len() > T::MaxDeviceIdLength::get() as usize {
                        return InvalidTransaction::Custom(1).into();
                    }
                    if proof_hash.len() > T::MaxProofLength::get() as usize {
                        return InvalidTransaction::Custom(2).into();
                    }
                    if window_start >= window_end {
                        return InvalidTransaction::Custom(3).into();
                    }

                    // Provides tag ties to (device_id, proof_hash) so the same
                    // proof cannot sit in the pool twice.
                    ValidTransaction::with_tag_prefix("TelemetryProof")
                        .priority(100)
                        .longevity(5)
                        .and_provides((device_id.clone(), proof_hash.clone()))
                        .propagate(false)
                        .build()
                }
                Call::submit_batch_proofs_unsigned { proofs } => {
                    if proofs.is_empty() {
                        return InvalidTransaction::Custom(4).into();
                    }
                    if proofs.len() > T::MaxBatchSize::get() as usize {
                        return InvalidTransaction::Custom(5).into();
                    }

                    // Build a deterministic provides tag from the first and last
                    // entries so identical batches are deduplicated in the pool.
                    let tag: (Vec<u8>, Vec<u8>) = (
                        proofs.first().map(|p| p.0.clone()).unwrap_or_default(),
                        proofs.last().map(|p| p.1.clone()).unwrap_or_default(),
                    );

                    ValidTransaction::with_tag_prefix("TelemetryProofBatch")
                        .priority(100)
                        .longevity(5)
                        .and_provides(tag)
                        .propagate(false)
                        .build()
                }
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}
