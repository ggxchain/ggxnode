use super::{pallet::Error, *};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use mock::*;

#[test]
fn test_create_id() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Psp37::create_id(RuntimeOrigin::signed(ALICE), ALICE),
			Error::<Test>::DefaultItemIdNotExist
		);

		assert_ok!(Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0));

		assert_ok!(Psp37::create_id(RuntimeOrigin::signed(ALICE), ALICE));
	})
}

#[test]
fn test_mint() {
	new_test_ext().execute_with(|| {
		assert_ok!(Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0));

		assert_ok!(Psp37::create_id(RuntimeOrigin::signed(ALICE), ALICE));
		assert_ok!(Psp37::mint(RuntimeOrigin::signed(ALICE), 0, ALICE));
	})
}

#[test]
fn test_approve() {
	new_test_ext().execute_with(|| {
		assert_ok!(Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0));

		assert_ok!(Psp37::create_id(RuntimeOrigin::signed(ALICE), ALICE));
		assert_ok!(Psp37::mint(RuntimeOrigin::signed(ALICE), 0, ALICE));

		assert_noop!(
			Psp37::approve(RuntimeOrigin::signed(CHARLIE), BOB, 0, 1),
			pallet_nfts::Error::<Test>::NoPermission
		);

		assert_ok!(Psp37::approve(RuntimeOrigin::signed(ALICE), BOB, 0, 1));
	})
}

#[test]
fn test_transfer() {
	new_test_ext().execute_with(|| {
		assert_ok!(Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0));

		assert_ok!(Psp37::create_id(RuntimeOrigin::signed(ALICE), ALICE));
		assert_ok!(Psp37::mint(RuntimeOrigin::signed(ALICE), 0, ALICE));

		assert_ok!(Psp37::transfer(
			RuntimeOrigin::signed(ALICE),
			BOB,
			0,
			0,
			vec![]
		));
	})
}

#[test]
fn test_transfer_from() {
	new_test_ext().execute_with(|| {
		assert_ok!(Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0));

		assert_ok!(Psp37::create_id(RuntimeOrigin::signed(ALICE), ALICE));
		assert_ok!(Psp37::mint(RuntimeOrigin::signed(ALICE), 0, ALICE));

		assert_ok!(Psp37::approve(RuntimeOrigin::signed(ALICE), BOB, 0, 1));

		assert_noop!(
			Psp37::transfer_from(RuntimeOrigin::signed(CHARLIE), ALICE, BOB, 0, 1, vec![]),
			Error::<Test>::FromIdNotEquOrigin
		);

		assert_noop!(
			Psp37::transfer_from(RuntimeOrigin::signed(CHARLIE), CHARLIE, BOB, 0, 1, vec![]),
			pallet_nfts::Error::<Test>::NoPermission
		);

		assert_ok!(Psp37::transfer_from(
			RuntimeOrigin::signed(BOB),
			BOB,
			CHARLIE,
			0,
			1,
			vec![]
		));
	})
}

#[test]
fn test_set_metadata() {
	new_test_ext().execute_with(|| {
		assert_ok!(Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0));

		assert_ok!(Psp37::create_id(RuntimeOrigin::signed(ALICE), ALICE));
		assert_ok!(Psp37::mint(RuntimeOrigin::signed(ALICE), 0, ALICE));

		let data: BoundedVec<u8, AssetsStringLimit> = vec![1, 2, 3].try_into().unwrap();

		assert_ok!(Psp37::set_metadata(RuntimeOrigin::signed(ALICE), 0, data));
	})
}

#[test]
fn test_set_default_item_id() {
	new_test_ext().execute_with(|| {
		assert_ok!(Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0));

		let item_id = DefaultItemId::<Test>::get();
		assert_eq!(item_id, Some(0));

		assert_noop!(
			Psp37::init_default_item_id(RuntimeOrigin::signed(ALICE), 0),
			Error::<Test>::DefaultItemIdHadInited
		);
	})
}
