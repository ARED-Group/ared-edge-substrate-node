//! Benchmarking setup for pallet-carbon-credits

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn issue_credits() {
        let caller: T::AccountId = whitelisted_caller();
        let device_id = vec![0u8; 36];
        let energy_kwh = 100u64;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), device_id, energy_kwh);
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
}
