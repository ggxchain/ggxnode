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
			Dex::make_order(RuntimeOrigin::signed(1), 777, 777, 1, 200, OrderType::SELL),
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
				order_type: OrderType::SELL,
				unfilled_offered: 1,
				unfilled_requested: 200,
				order_status: OrderStatus::Pending,
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
				order_type: OrderType::BUY,
				unfilled_offered: 1,
				unfilled_requested: 200,
				order_status: OrderStatus::Pending,
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
				order_type: OrderType::SELL,
				unfilled_offered: 1,
				unfilled_requested: 200,
				order_status: OrderStatus::Pending,
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
			OrderType::BUY
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

#[test]
fn test_offchain_worker_order_matching() {
	/*
	@@ input orders
	//order_index  order_type price amount - offered_amount requested_amount
	0 buy  208234 1  - 208234 1
	1 sell 208760 2  - 2 417520
	2 buy  208780 1  - 208780 1
	3 buy  208501 5  - 1042505 5
	4 sell 208802 3  - 3 626406
	5 sell 208760 6  - 6 1252560
	6 buy  208111 7  - 1456777 7
	7 buy  208600 3  - 625800 3
	8 buy  208833 1  - 208833 1
	9 sell 208654 2  - 2 417308
	10 sell 208655 5  - 5 1043275
	11 buy  208655 3  - 625965 3

	@@ finnal order_book and last_trade_price - offered_amount requested_amount:
	208802 3 - SELL 3 626406
	208760 6 - SELL 6 1252560
	208655 4 - SELL 4 834620
	---------
	208655
	---------
	208600 3 - BUY 625800 3
	208501 5 - BUY 1042505 5
	208234 1 - BUY 208234 1
	208111 7 - BUY 1456777 7
	*/
	let mut ext = new_test_ext();

	ext.execute_with(|| add_blocks(1));
	ext.persist_offchain_overlay();

	register_offchain_ext(&mut ext);
	ext.execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 777, 1_000_000_000));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(1), 888, 1_000_000_000));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			208234,
			1,
			OrderType::BUY
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			2,
			417520,
			OrderType::SELL
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			208780,
			1,
			OrderType::BUY
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			1042505,
			5,
			OrderType::BUY
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			3,
			626406,
			OrderType::SELL
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			6,
			1252560,
			OrderType::SELL
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			1456777,
			7,
			OrderType::BUY
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			625800,
			3,
			OrderType::BUY
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			208833,
			1,
			OrderType::BUY
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			2,
			417308,
			OrderType::SELL
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			5,
			1043275,
			OrderType::SELL
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(1),
			777,
			888,
			625965,
			3,
			OrderType::BUY
		));

		add_blocks(6);

		//order_book  price=> (total_offered_amount, total_requested_amount)
		let mut sell_order_book = BTreeMap::new();
		let mut buy_order_book = BTreeMap::new();

		for (_, order) in Orders::<Test>::iter() {
			if order.order_status != OrderStatus::FullyFilled {
				if order.order_type == OrderType::SELL {
					let price = order.amout_requested / order.amount_offered;
					if !buy_order_book.contains_key(&price) {
						buy_order_book.insert(price, (order.amount_offered, order.amout_requested));
					} else {
						let v = buy_order_book.get(&price).unwrap();

						buy_order_book.insert(
							price,
							(v.0 + order.amount_offered, v.1 + order.amout_requested),
						);
					}
				} else {
					let price = order.amount_offered / order.amout_requested;

					if !sell_order_book.contains_key(&price) {
						sell_order_book
							.insert(price, (order.amount_offered, order.amout_requested));
					} else {
						let v = sell_order_book.get(&price).unwrap();
						sell_order_book.insert(
							price,
							(v.0 + order.amount_offered, v.1 + order.amout_requested),
						);
					}
				}
			}
		}

		assert_eq!(sell_order_book.len(), 3);
		assert_eq!(sell_order_book.get(&208802).unwrap(), &(3, 626406));
		assert_eq!(sell_order_book.get(&208760).unwrap(), &(6, 1252560));
		assert_eq!(sell_order_book.get(&208655).unwrap(), &(4, 834620));

		assert_eq!(buy_order_book.len(), 4);
		assert_eq!(buy_order_book.get(&208600).unwrap(), &(625800, 3));
		assert_eq!(buy_order_book.get(&208501).unwrap(), &(1042505, 5));
		assert_eq!(buy_order_book.get(&208234).unwrap(), &(208234, 1));
		assert_eq!(buy_order_book.get(&208111).unwrap(), &(1456777, 7));
	})
}
