use super::{pallet::Error, Event, *};
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
				asset_id: 777,
				amount: 10,
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
				asset_id: 777,
				amount: 10,
			}
		);

		assert_noop!(
			Dex::withdraw(RuntimeOrigin::signed(1), 777, 11),
			Error::<Test>::TokenBalanceOverflow
		);

		assert_ok!(Dex::withdraw(RuntimeOrigin::signed(1), 777, 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(1, 777),
			TokenInfo {
				asset_id: 777,
				amount: 0,
			}
		);
	})
}

#[test]
fn test_make_order() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 100));

		assert_noop!(
			Dex::make_order(RuntimeOrigin::signed(1), 777, 777, 1, 200, OrderType::SELL),
			Error::<Test>::PairAssetIdMustNotEqual
		);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			1,
			200,
			OrderType::SELL
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: 1,
				pair: (777, 888),
				timestamp: 0,
				amount_offered: 1,
				amout_requested: 200,
				order_type: OrderType::SELL
			})
		);

		assert_eq!(UserOrders::<Test>::get(1, 0), ());

		assert_eq!(PairOrders::<Test>::get((777, 888)), vec![0]);
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
			OrderType::SELL
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: 1,
				pair: (777, 888),
				timestamp: 0,
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
			OrderType::SELL
		));

		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::UserAssetNotExist
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 888, 100));
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::TokenBalanceOverflow
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 888, 200));

		assert_eq!(UserTokenInfoes::<Test>::get(1, 777).amount, 100);
		assert_eq!(UserTokenInfoes::<Test>::get(1, 888).amount, 0);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 777).amount, 0);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 888).amount, 300);

		assert_ok!(Dex::take_order(RuntimeOrigin::signed(2), 0));

		assert_eq!(UserTokenInfoes::<Test>::get(1, 777).amount, 99);
		assert_eq!(UserTokenInfoes::<Test>::get(1, 888).amount, 200);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 777).amount, 1);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 888).amount, 100);
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
			OrderType::BUY
		));

		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::UserAssetNotExist
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 777, 1));
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(2), 0),
			Error::<Test>::TokenBalanceOverflow
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(2), 777, 2));

		assert_eq!(UserTokenInfoes::<Test>::get(1, 777).amount, 0);
		assert_eq!(UserTokenInfoes::<Test>::get(1, 888).amount, 200);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 777).amount, 3);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 888).amount, 0);

		assert_ok!(Dex::take_order(RuntimeOrigin::signed(2), 0));

		assert_eq!(UserTokenInfoes::<Test>::get(1, 777).amount, 2);
		assert_eq!(UserTokenInfoes::<Test>::get(1, 888).amount, 0);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 777).amount, 1);
		assert_eq!(UserTokenInfoes::<Test>::get(2, 888).amount, 200);
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
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 888, 200));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 200));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			100,
			1,
			OrderType::BUY
		));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			200,
			2,
			OrderType::BUY
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			888,
			999,
			200,
			2,
			OrderType::SELL
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
