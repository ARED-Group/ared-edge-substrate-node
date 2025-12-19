//! Unit tests for the Telemetry Proofs pallet.

use crate::{self as pallet_telemetry_proofs, *};
use frame_support::{
    assert_noop, assert_ok,
    traits::{ConstU32, ConstU64},
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
        TelemetryProofs: pallet_telemetry_proofs,
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

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxDeviceIdLength = ConstU32<64>;
    type MaxProofLength = ConstU32<128>;
    type MaxBatchSize = ConstU32<100>;
    type MaxProofsPerDevice = ConstU32<1000>;
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

fn proof_hash(hash: &str) -> Vec<u8> {
    hash.as_bytes().to_vec()
}

#[test]
fn submit_proof_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let dev_id = device_id("device-001");
        let hash = proof_hash("abc123hash");

        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            hash.clone(),
            10,   // record_count
            1000, // window_start
            2000, // window_end
        ));

        // Check proof count
        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.try_into().unwrap();
        assert_eq!(TelemetryProofs::proof_count(&bounded_dev_id), 1);

        // Check total proofs
        assert_eq!(TelemetryProofs::total_proofs(), 1);

        // Check proof exists
        let proof = TelemetryProofs::proofs(&bounded_dev_id, 0);
        assert!(proof.is_some());
        let metadata = proof.unwrap();
        assert_eq!(metadata.record_count, 10);
        assert_eq!(metadata.window_start, 1000);
        assert_eq!(metadata.window_end, 2000);
    });
}

#[test]
fn submit_proof_rejects_duplicate_in_same_block() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let dev_id = device_id("device-001");
        let hash1 = proof_hash("hash1");
        let hash2 = proof_hash("hash2");

        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            hash1,
            10,
            1000,
            2000,
        ));

        // Second proof in same block should fail
        assert_noop!(
            TelemetryProofs::submit_proof(RuntimeOrigin::signed(1), dev_id, hash2, 10, 2000, 3000,),
            Error::<Test>::ProofAlreadyExists
        );
    });
}

#[test]
fn submit_proof_allows_same_device_different_blocks() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");

        System::set_block_number(1);
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            proof_hash("hash1"),
            10,
            1000,
            2000,
        ));

        System::set_block_number(2);
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            proof_hash("hash2"),
            10,
            2000,
            3000,
        ));

        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.try_into().unwrap();
        assert_eq!(TelemetryProofs::proof_count(&bounded_dev_id), 2);
    });
}

#[test]
fn submit_proof_rejects_invalid_time_window() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // window_start >= window_end should fail
        assert_noop!(
            TelemetryProofs::submit_proof(
                RuntimeOrigin::signed(1),
                device_id("device-001"),
                proof_hash("hash"),
                10,
                2000, // start
                1000, // end (before start)
            ),
            Error::<Test>::InvalidTimeWindow
        );

        // Equal values should also fail
        assert_noop!(
            TelemetryProofs::submit_proof(
                RuntimeOrigin::signed(1),
                device_id("device-001"),
                proof_hash("hash"),
                10,
                1000,
                1000,
            ),
            Error::<Test>::InvalidTimeWindow
        );
    });
}

#[test]
fn submit_proof_rejects_too_long_device_id() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let long_id = vec![b'a'; 100]; // Exceeds MaxDeviceIdLength (64)

        assert_noop!(
            TelemetryProofs::submit_proof(
                RuntimeOrigin::signed(1),
                long_id,
                proof_hash("hash"),
                10,
                1000,
                2000,
            ),
            Error::<Test>::DeviceIdTooLong
        );
    });
}

#[test]
fn submit_batch_proofs_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proofs = vec![
            (
                device_id("device-001"),
                proof_hash("hash1"),
                10,
                1000u64,
                2000u64,
            ),
            (
                device_id("device-002"),
                proof_hash("hash2"),
                20,
                2000u64,
                3000u64,
            ),
            (
                device_id("device-003"),
                proof_hash("hash3"),
                30,
                3000u64,
                4000u64,
            ),
        ];

        assert_ok!(TelemetryProofs::submit_batch_proofs(
            RuntimeOrigin::signed(1),
            proofs,
        ));

        // Check total proofs
        assert_eq!(TelemetryProofs::total_proofs(), 3);

        // Check each device
        let dev1: BoundedVec<u8, ConstU32<64>> = device_id("device-001").try_into().unwrap();
        let dev2: BoundedVec<u8, ConstU32<64>> = device_id("device-002").try_into().unwrap();
        let dev3: BoundedVec<u8, ConstU32<64>> = device_id("device-003").try_into().unwrap();

        assert_eq!(TelemetryProofs::proof_count(&dev1), 1);
        assert_eq!(TelemetryProofs::proof_count(&dev2), 1);
        assert_eq!(TelemetryProofs::proof_count(&dev3), 1);
    });
}

#[test]
fn submit_batch_proofs_skips_invalid_entries() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let proofs = vec![
            (
                device_id("device-001"),
                proof_hash("hash1"),
                10,
                1000u64,
                2000u64,
            ), // Valid
            (
                device_id("device-002"),
                proof_hash("hash2"),
                20,
                3000u64,
                2000u64,
            ), // Invalid window
            (
                device_id("device-003"),
                proof_hash("hash3"),
                30,
                3000u64,
                4000u64,
            ), // Valid
        ];

        assert_ok!(TelemetryProofs::submit_batch_proofs(
            RuntimeOrigin::signed(1),
            proofs,
        ));

        // Only 2 valid proofs should be stored
        assert_eq!(TelemetryProofs::total_proofs(), 2);
    });
}

#[test]
fn submit_batch_proofs_rejects_empty_batch() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_noop!(
            TelemetryProofs::submit_batch_proofs(RuntimeOrigin::signed(1), vec![],),
            Error::<Test>::EmptyBatch
        );
    });
}

#[test]
fn verify_proof_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let dev_id = device_id("device-001");
        let hash = proof_hash("abc123hash");

        // Submit a proof first
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            hash.clone(),
            10,
            1000,
            2000,
        ));

        // Verify existing proof
        assert_ok!(TelemetryProofs::verify_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            hash,
        ));

        // Verify non-existent proof
        assert_ok!(TelemetryProofs::verify_proof(
            RuntimeOrigin::signed(1),
            dev_id,
            proof_hash("nonexistent"),
        ));
    });
}

#[test]
fn proof_exists_helper_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let dev_id = device_id("device-001");
        let hash = proof_hash("abc123hash");

        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            hash.clone(),
            10,
            1000,
            2000,
        ));

        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.try_into().unwrap();
        let bounded_hash: BoundedVec<u8, ConstU32<128>> = hash.try_into().unwrap();

        assert!(TelemetryProofs::proof_exists(
            &bounded_dev_id,
            &bounded_hash
        ));

        let nonexistent: BoundedVec<u8, ConstU32<128>> =
            proof_hash("nonexistent").try_into().unwrap();
        assert!(!TelemetryProofs::proof_exists(
            &bounded_dev_id,
            &nonexistent
        ));
    });
}

#[test]
fn get_proofs_in_window_works() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");

        // Submit proofs at different blocks with different windows
        System::set_block_number(1);
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            proof_hash("hash1"),
            10,
            1000,
            2000,
        ));

        System::set_block_number(2);
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            proof_hash("hash2"),
            10,
            2000,
            3000,
        ));

        System::set_block_number(3);
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            proof_hash("hash3"),
            10,
            5000,
            6000,
        ));

        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.try_into().unwrap();

        // Query window 1000-3000 should return first two proofs
        let proofs = TelemetryProofs::get_proofs_in_window(&bounded_dev_id, 1000, 3000);
        assert_eq!(proofs.len(), 2);

        // Query window 4000-7000 should return last proof
        let proofs = TelemetryProofs::get_proofs_in_window(&bounded_dev_id, 4000, 7000);
        assert_eq!(proofs.len(), 1);

        // Query window 0-10000 should return all proofs
        let proofs = TelemetryProofs::get_proofs_in_window(&bounded_dev_id, 0, 10000);
        assert_eq!(proofs.len(), 3);
    });
}

#[test]
fn latest_proof_block_updated() {
    new_test_ext().execute_with(|| {
        let dev_id = device_id("device-001");
        let bounded_dev_id: BoundedVec<u8, ConstU32<64>> = dev_id.clone().try_into().unwrap();

        // Initially no latest block
        assert!(TelemetryProofs::latest_proof_block(&bounded_dev_id).is_none());

        System::set_block_number(5);
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id.clone(),
            proof_hash("hash1"),
            10,
            1000,
            2000,
        ));

        assert_eq!(
            TelemetryProofs::latest_proof_block(&bounded_dev_id),
            Some(5)
        );

        System::set_block_number(10);
        assert_ok!(TelemetryProofs::submit_proof(
            RuntimeOrigin::signed(1),
            dev_id,
            proof_hash("hash2"),
            10,
            2000,
            3000,
        ));

        assert_eq!(
            TelemetryProofs::latest_proof_block(&bounded_dev_id),
            Some(10)
        );
    });
}
