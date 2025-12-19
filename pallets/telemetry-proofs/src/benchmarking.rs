//! Benchmarking setup for pallet-telemetry-proofs

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_proof() {
        let caller: T::AccountId = whitelisted_caller();
        let device_id = vec![0u8; 36];
        let proof_hash = vec![0u8; 32];
        let record_count = 10u32;
        let window_start = 0u64;
        let window_end = 3600u64;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            device_id,
            proof_hash,
            record_count,
            window_start,
            window_end,
        );
    }

    #[benchmark]
    fn submit_batch_proofs(n: Linear<1, 100>) {
        let caller: T::AccountId = whitelisted_caller();
        let proofs: Vec<(Vec<u8>, Vec<u8>, u32, u64, u64)> = (0..n)
            .map(|i| (vec![i as u8; 36], vec![i as u8; 32], 10u32, 0u64, 3600u64))
            .collect();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), proofs);
    }

    #[benchmark]
    fn verify_proof() {
        let caller: T::AccountId = whitelisted_caller();
        let device_id = vec![0u8; 36];
        let proof_hash = vec![0u8; 32];

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), device_id, proof_hash);
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
