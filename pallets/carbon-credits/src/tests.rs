//! Unit tests for the Carbon Credits pallet.

use crate::{self as pallet_carbon_credits, *};
use frame_support::{
    assert_noop, assert_ok,
    traits::{ConstU32, ConstU64, ConstU128},
    BoundedVec,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        CarbonCredits: pallet_carbon_credits,
    }
);

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

/// Default emission factor: 1.5 kg CO2/kWh (scaled by 1000 = 1500)
pub struct DefaultEmissionFactor;
impl frame_support::traits::Get<u32> for DefaultEmissionFactor {
    fn get() -> u32 {
        1500 // 1.5 kg CO2/kWh
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxDeviceIdLength = ConstU32<64>;
    type CreditsPerTonCO2 = ConstU128<1000>; // 1000 credits per ton
    type DefaultEmissionFactor = DefaultEmissionFactor;
    type MinClaimableEnergy = ConstU128<1000>; // 1 kWh minimum
    type MaxIssuanceRecords = ConstU32<1000>;
}

fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    t.into()
}

fn device_id(id: &str) -> Vec<u8> {
    id.as_bytes().to_vec()
}

#[test]
fn record_energy_works() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            5000, // 5 kWh
            None,
        ));
        
        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.try_into().unwrap();
        assert_eq!(CarbonCredits::energy_accumulated(&bounded_dev_id), 5000);
        assert_eq!(CarbonCredits::total_energy(&bounded_dev_id), 5000);
        assert_eq!(CarbonCredits::active_device_count(), 1);
    });
}

#[test]
fn record_energy_accumulates() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            3000,
            None,
        ));
        
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            2000,
            None,
        ));
        
        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.try_into().unwrap();
        assert_eq!(CarbonCredits::energy_accumulated(&bounded_dev_id), 5000);
        assert_eq!(CarbonCredits::total_energy(&bounded_dev_id), 5000);
        
        // Device count should still be 1
        assert_eq!(CarbonCredits::active_device_count(), 1);
    });
}

#[test]
fn claim_credits_works() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        
        // Record 10 kWh (10000 Wh)
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            10_000,
            None,
        ));
        
        // Claim credits
        assert_ok!(CarbonCredits::claim_credits(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
        ));
        
        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.try_into().unwrap();
        
        // Verify accumulated energy is reset
        assert_eq!(CarbonCredits::energy_accumulated(&bounded_dev_id), 0);
        
        // Calculate expected credits:
        // energy_kwh = 10000 / 1000 = 10 kWh
        // co2_avoided_kg = 10 * 1500 / 1000 = 15 kg
        // credits = 15 * 1000 / 1000 = 15 credits
        assert_eq!(CarbonCredits::credits_balance(&bounded_dev_id), 15);
        assert_eq!(CarbonCredits::total_credits_issued(), 15);
        assert_eq!(CarbonCredits::total_co2_avoided(), 15);
        assert_eq!(CarbonCredits::issuance_count(&bounded_dev_id), 1);
    });
}

#[test]
fn claim_credits_requires_minimum_energy() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        
        // Record less than minimum (1000 Wh)
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            500, // 0.5 kWh
            None,
        ));
        
        // Claim should fail
        assert_noop!(
            CarbonCredits::claim_credits(
                RuntimeOrigin::signed(1),
                dev_id,
            ),
            Error::<Test>::EnergyBelowMinimum
        );
    });
}

#[test]
fn claim_credits_fails_with_no_energy() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        
        assert_noop!(
            CarbonCredits::claim_credits(
                RuntimeOrigin::signed(1),
                dev_id,
            ),
            Error::<Test>::NoCreditsAvailable
        );
    });
}

#[test]
fn transfer_credits_works() {
    new_test_ext().execute_with(|| {
        let dev1 = device_id("device-001");
        let dev2 = device_id("device-002");
        
        // Record and claim for device 1
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev1.clone(),
            100_000, // 100 kWh
            None,
        ));
        assert_ok!(CarbonCredits::claim_credits(
            RuntimeOrigin::signed(1),
            dev1.clone(),
        ));
        
        let bounded_dev1: BoundedVec<u8, ConstU32<64>> = dev1.clone().try_into().unwrap();
        let bounded_dev2: BoundedVec<u8, ConstU32<64>> = dev2.clone().try_into().unwrap();
        
        let initial_balance = CarbonCredits::credits_balance(&bounded_dev1);
        assert!(initial_balance > 0);
        
        // Transfer half
        let transfer_amount = initial_balance / 2;
        assert_ok!(CarbonCredits::transfer_credits(
            RuntimeOrigin::signed(1),
            dev1,
            dev2,
            transfer_amount,
        ));
        
        assert_eq!(
            CarbonCredits::credits_balance(&bounded_dev1),
            initial_balance - transfer_amount
        );
        assert_eq!(CarbonCredits::credits_balance(&bounded_dev2), transfer_amount);
    });
}

#[test]
fn transfer_credits_fails_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let dev1 = device_id("device-001");
        let dev2 = device_id("device-002");
        
        // Try to transfer without any credits
        assert_noop!(
            CarbonCredits::transfer_credits(
                RuntimeOrigin::signed(1),
                dev1,
                dev2,
                100,
            ),
            Error::<Test>::InsufficientCredits
        );
    });
}

#[test]
fn transfer_credits_fails_same_device() {
    new_test_ext().execute_with(|| {
        let dev1 = device_id("device-001");
        
        assert_noop!(
            CarbonCredits::transfer_credits(
                RuntimeOrigin::signed(1),
                dev1.clone(),
                dev1,
                100,
            ),
            Error::<Test>::SameDeviceTransfer
        );
    });
}

#[test]
fn withdraw_credits_works() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        let account: u64 = 42;
        
        // Record and claim
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            100_000,
            None,
        ));
        assert_ok!(CarbonCredits::claim_credits(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
        ));
        
        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.clone().try_into().unwrap();
        let device_balance = CarbonCredits::credits_balance(&bounded_dev_id);
        
        // Withdraw to account
        let withdraw_amount = device_balance / 2;
        assert_ok!(CarbonCredits::withdraw_credits(
            RuntimeOrigin::signed(account),
            dev_id,
            withdraw_amount,
        ));
        
        assert_eq!(
            CarbonCredits::credits_balance(&bounded_dev_id),
            device_balance - withdraw_amount
        );
        assert_eq!(CarbonCredits::account_credits(&account), withdraw_amount);
    });
}

#[test]
fn set_emission_factor_works() {
    new_test_ext().execute_with(|| {
        let initial_factor = CarbonCredits::emission_factor();
        assert_eq!(initial_factor, 1500); // Default
        
        // Update factor (requires root)
        assert_ok!(CarbonCredits::set_emission_factor(
            RuntimeOrigin::root(),
            2000, // 2.0 kg CO2/kWh
        ));
        
        assert_eq!(CarbonCredits::emission_factor(), 2000);
    });
}

#[test]
fn set_emission_factor_requires_root() {
    new_test_ext().execute_with(|| {
        // Non-root should fail
        assert_noop!(
            CarbonCredits::set_emission_factor(
                RuntimeOrigin::signed(1),
                2000,
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn set_emission_factor_rejects_zero() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            CarbonCredits::set_emission_factor(
                RuntimeOrigin::root(),
                0,
            ),
            Error::<Test>::InvalidEmissionFactor
        );
    });
}

#[test]
fn calculate_credits_helper_works() {
    new_test_ext().execute_with(|| {
        // With default emission factor of 1500 (1.5 kg/kWh)
        // and 1000 credits per ton:
        // 100 kWh = 100 * 1.5 = 150 kg CO2 = 0.15 ton = 150 credits
        let credits = CarbonCredits::calculate_credits(100_000); // 100 kWh in Wh
        assert_eq!(credits, 150);
        
        // 1000 kWh = 1000 * 1.5 = 1500 kg = 1.5 ton = 1500 credits
        let credits = CarbonCredits::calculate_credits(1_000_000); // 1000 kWh
        assert_eq!(credits, 1500);
    });
}

#[test]
fn get_stats_works() {
    new_test_ext().execute_with(|| {
        let dev1 = device_id("device-001");
        let dev2 = device_id("device-002");
        
        // Record energy for two devices
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev1.clone(),
            50_000,
            None,
        ));
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev2.clone(),
            50_000,
            None,
        ));
        
        // Claim for both
        assert_ok!(CarbonCredits::claim_credits(RuntimeOrigin::signed(1), dev1));
        assert_ok!(CarbonCredits::claim_credits(RuntimeOrigin::signed(1), dev2));
        
        let (total_credits, total_co2, active_devices) = CarbonCredits::get_stats();
        
        assert!(total_credits > 0);
        assert!(total_co2 > 0);
        assert_eq!(active_devices, 2);
    });
}

#[test]
fn total_energy_persists_after_claim() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        
        // Record 10 kWh
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            10_000,
            None,
        ));
        
        // Claim
        assert_ok!(CarbonCredits::claim_credits(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
        ));
        
        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.clone().try_into().unwrap();
        
        // Accumulated should be 0, but total should remain
        assert_eq!(CarbonCredits::energy_accumulated(&bounded_dev_id), 0);
        assert_eq!(CarbonCredits::total_energy(&bounded_dev_id), 10_000);
        
        // Record more
        assert_ok!(CarbonCredits::record_energy(
            RuntimeOrigin::signed(1),
            dev_id,
            5_000,
            None,
        ));
        
        assert_eq!(CarbonCredits::energy_accumulated(&bounded_dev_id), 5_000);
        assert_eq!(CarbonCredits::total_energy(&bounded_dev_id), 15_000);
    });
}
