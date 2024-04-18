use super::{pallet::Error, *};
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn test_deposit() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::deposit(RuntimeOrigin::signed(1), 666, 10),
			Error::<Test>::AssetIdNotInTokenIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_withdraw() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::withdraw(RuntimeOrigin::signed(1), 777, 10),
			Error::<Test>::AssetIdNotInTokenInfoes
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_noop!(
			Dex::withdraw(RuntimeOrigin::signed(1), 777, 11),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::withdraw(RuntimeOrigin::signed(1), 777, 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 0,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_deposit_native() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(1), 9000);

		assert_ok!(Dex::deposit_native(RuntimeOrigin::signed(1), 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, NativeAssetId::<Test>::get()),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_eq!(Balances::free_balance(1), 8990);
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, NativeAssetId::<Test>::get()),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_withdraw_native() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(1), 9000);

		assert_noop!(
			Dex::withdraw_native(RuntimeOrigin::signed(1), 10),
			Error::<Test>::AssetIdNotInTokenInfoes
		);

		assert_ok!(Dex::deposit_native(RuntimeOrigin::signed(1), 10));

		assert_eq!(Balances::free_balance(1), 8990);
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, NativeAssetId::<Test>::get()),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_noop!(
			Dex::withdraw_native(RuntimeOrigin::signed(1), 11),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::withdraw_native(RuntimeOrigin::signed(1), 10));

		assert_eq!(Balances::free_balance(1), 9000);
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, NativeAssetId::<Test>::get()),
			TokenInfo {
				amount: 0,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_make_order() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 100));

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(1),
				777,
				777,
				1,
				200,
				OrderType::SELL,
				1000
			),
			Error::<Test>::PairAssetIdMustNotEqual
		);

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 100,
				reserved: 0,
			}
		);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			1,
			200,
			OrderType::SELL,
			1000
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: 1,
				pair: (777, 888),
				expiration_block: 1000,
				amount_offered: 1,
				amout_requested: 200,
				order_type: OrderType::SELL
			})
		);

		assert_eq!(UserOrders::<Test>::get(1, 0), ());

		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![0]);

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 99,
				reserved: 1,
			}
		);
	})
}

#[test]
fn test_make_order_asset_id_1_gt_asset_id_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 100));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 888, 200));

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 100,
				reserved: 0,
			}
		);

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 888),
			TokenInfo {
				amount: 200,
				reserved: 0,
			}
		);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			888,
			777,
			1,
			200,
			OrderType::SELL,
			1000
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: 1,
				pair: (777, 888),
				expiration_block: 1000,
				amount_offered: 1,
				amout_requested: 200,
				order_type: OrderType::BUY
			})
		);

		assert_eq!(UserOrders::<Test>::get(1, 0), ());

		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![0]);

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 888),
			TokenInfo {
				amount: 199,
				reserved: 1,
			}
		);
	})
}

#[test]
fn test_cancel_order() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::cancel_order(RuntimeOrigin::signed(1), 0),
			Error::<Test>::InvalidOrderIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 100));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			1,
			200,
			OrderType::SELL,
			1000
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: 1,
				pair: (777, 888),
				expiration_block: 1000,
				amount_offered: 1,
				amout_requested: 200,
				order_type: OrderType::SELL
			})
		);
		assert_eq!(UserOrders::<Test>::contains_key(1, 0), true);
		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![0]);

		assert_noop!(
			Dex::cancel_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NotOwner
		);

		assert_ok!(Dex::cancel_order(RuntimeOrigin::signed(1), 0));

		assert_eq!(Orders::<Test>::get(0), None);
		assert_eq!(UserOrders::<Test>::contains_key(1, 1), false);
		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![]);
	})
}

#[test]
fn test_take_order_sell() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 100));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			1,
			200,
			OrderType::SELL,
			1000
		));

		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::UserAssetNotExist
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 888, 100));
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 888, 200));

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 99,
				reserved: 1,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 888),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 777),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 888),
			TokenInfo {
				amount: 300,
				reserved: 0,
			}
		);

		assert_ok!(Dex::take_order(RuntimeOrigin::signed(2), 0));

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 99,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 888),
			TokenInfo {
				amount: 200,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 777),
			TokenInfo {
				amount: 1,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 888),
			TokenInfo {
				amount: 100,
				reserved: 0,
			}
		);
	})
}

#[test]
fn test_take_order_buy() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::InvalidOrderIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 888, 200));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			200,
			2,
			OrderType::BUY,
			1000
		));

		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::UserAssetNotExist
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 777, 1));
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 777, 2));

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 888),
			TokenInfo {
				amount: 0,
				reserved: 200,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 777),
			TokenInfo {
				amount: 3,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 888),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);

		assert_ok!(Dex::take_order(RuntimeOrigin::signed(2), 0));

		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				amount: 2,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 888),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 777),
			TokenInfo {
				amount: 1,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(2, 888),
			TokenInfo {
				amount: 200,
				reserved: 0,
			}
		);
	})
}

#[test]
fn test_make_cancel_take_order_buy() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::InvalidOrderIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 999, 200));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 888, 500));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 200));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			100,
			1,
			OrderType::BUY,
			1000
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			200,
			2,
			OrderType::BUY,
			1000
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			888,
			999,
			200,
			2,
			OrderType::SELL,
			1000
		));

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 999, 300));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 888, 300));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 777, 300));

		assert_eq!(Orders::<Test>::contains_key(0), true);
		assert_eq!(Orders::<Test>::contains_key(1), true);
		assert_eq!(Orders::<Test>::contains_key(2), true);
		assert_eq!(UserOrders::<Test>::contains_key(1, 0), true);
		assert_eq!(UserOrders::<Test>::contains_key(1, 1), true);
		assert_eq!(UserOrders::<Test>::contains_key(1, 2), true);
		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![0, 1]);
		assert_eq!(PairOrders::<Test>::get((888, 999)), vec![2]);

		assert_ok!(Dex::cancel_order(RuntimeOrigin::signed(1), 1));
		assert_ok!(Dex::take_order(RuntimeOrigin::signed(2), 0));

		assert_eq!(Orders::<Test>::contains_key(0), false);
		assert_eq!(Orders::<Test>::contains_key(1), false);
		assert_eq!(Orders::<Test>::contains_key(2), true);
		assert_eq!(UserOrders::<Test>::contains_key(1, 0), false);
		assert_eq!(UserOrders::<Test>::contains_key(1, 1), false);
		assert_eq!(UserOrders::<Test>::contains_key(1, 2), true);
		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![]);
		assert_eq!(PairOrders::<Test>::get((888, 999)), vec![2]);
	})
}

#[test]
fn test_expiration_works_as_expected() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 888, 200));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			100,
			1,
			OrderType::BUY,
			10
		));

		assert_eq!(Orders::<Test>::contains_key(0), true);
		assert_eq!(UserOrders::<Test>::contains_key(1, 0), true);
		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![0]);
		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: 1,
				pair: (777, 888),
				expiration_block: 10,
				amount_offered: 100,
				amout_requested: 1,
				order_type: OrderType::BUY,
			}),
		);
		assert_eq!(OrderExpiration::<Test>::get(10), vec![0]);

		run_to_block(11);

		assert!(!Orders::<Test>::contains_key(0));
		assert!(!UserOrders::<Test>::contains_key(1, 0));
		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![]);
		assert_eq!(OrderExpiration::<Test>::get(10), vec![]);
	});
}

#[test]
fn fail_on_invalid_expiry() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 888, 200));
		run_to_block(5);

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(1),
				777,
				888,
				100,
				1,
				OrderType::BUY,
				3
			),
			Error::<Test>::ExpirationMustBeInFuture
		);
		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(1),
				777,
				888,
				100,
				1,
				OrderType::BUY,
				5
			),
			Error::<Test>::ExpirationMustBeInFuture
		);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			100,
			1,
			OrderType::BUY,
			6
		));
	});
}