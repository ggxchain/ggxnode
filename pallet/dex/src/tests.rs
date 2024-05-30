use super::{pallet::Error, *};
use frame_support::{assert_noop, assert_ok};
use mock::*;
use scale_info::prelude::collections::BTreeMap;
use sp_core::{
	offchain::{
		testing::{TestOffchainExt, TestTransactionPoolExt},
		OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
	},
	U256,
};

#[test]
fn test_deposit() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::deposit(RuntimeOrigin::signed(ALICE), DOT, 10),
			Error::<Test>::AssetIdNotInTokenIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), USDT, 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
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
			Dex::withdraw(RuntimeOrigin::signed(ALICE), USDT, 10),
			Error::<Test>::AssetIdNotInTokenInfoes
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), USDT, 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_noop!(
			Dex::withdraw(RuntimeOrigin::signed(ALICE), USDT, 11),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::withdraw(RuntimeOrigin::signed(ALICE), USDT, 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
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
		assert_eq!(Balances::free_balance(ALICE), 100_000_000_000);

		assert_ok!(Dex::deposit_native(RuntimeOrigin::signed(ALICE), 10));
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, NativeAssetId::<Test>::get()),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_eq!(Balances::free_balance(ALICE), 99999999990);
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, NativeAssetId::<Test>::get()),
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
		assert_eq!(Balances::free_balance(ALICE), 100_000_000_000);

		assert_noop!(
			Dex::withdraw_native(RuntimeOrigin::signed(ALICE), 10),
			Error::<Test>::AssetIdNotInTokenInfoes
		);

		assert_ok!(Dex::deposit_native(RuntimeOrigin::signed(ALICE), 10));

		assert_eq!(Balances::free_balance(ALICE), 99999999990);
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, NativeAssetId::<Test>::get()),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_noop!(
			Dex::withdraw_native(RuntimeOrigin::signed(ALICE), 11),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::withdraw_native(RuntimeOrigin::signed(ALICE), 10));

		assert_eq!(Balances::free_balance(ALICE), 100_000_000_000);
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, NativeAssetId::<Test>::get()),
			TokenInfo {
				amount: 0,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_deposit_erc20() {
	new_test_ext().execute_with(|| {
		deploy_contracts();

		assert_ok!(Dex::deposit(
			RuntimeOrigin::signed(ALICE),
			CurrencyId::Erc20(erc20_address()),
			10
		));

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, CurrencyId::Erc20(erc20_address()),),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_withdraw_erc20() {
	new_test_ext().execute_with(|| {
		deploy_contracts();

		assert_noop!(
			Dex::withdraw(
				RuntimeOrigin::signed(ALICE),
				CurrencyId::Erc20(erc20_address()),
				10
			),
			Error::<Test>::AssetIdNotInTokenInfoes
		);

		assert_ok!(Dex::deposit(
			RuntimeOrigin::signed(ALICE),
			CurrencyId::Erc20(erc20_address()),
			10
		));
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, CurrencyId::Erc20(erc20_address()),),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_noop!(
			Dex::withdraw(
				RuntimeOrigin::signed(ALICE),
				CurrencyId::Erc20(erc20_address()),
				11
			),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::withdraw(
			RuntimeOrigin::signed(ALICE),
			CurrencyId::Erc20(erc20_address()),
			10
		));
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, CurrencyId::Erc20(erc20_address()),),
			TokenInfo {
				amount: 0,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_deposit_erc1155() {
	new_test_ext().execute_with(|| {
		deploy_erc1155_contracts();

		assert_ok!(Dex::deposit(
			RuntimeOrigin::signed(ALICE),
			CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
			10
		));

		assert_eq!(
			UserTokenInfoes::<Test>::get(
				ALICE,
				CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
			),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);
	})
}

#[test]
fn test_withdraw_erc1155() {
	new_test_ext().execute_with(|| {
		deploy_erc1155_contracts();

		assert_noop!(
			Dex::withdraw(
				RuntimeOrigin::signed(ALICE),
				CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
				10
			),
			Error::<Test>::AssetIdNotInTokenInfoes
		);

		assert_ok!(Dex::deposit(
			RuntimeOrigin::signed(ALICE),
			CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
			10
		));
		assert_eq!(
			UserTokenInfoes::<Test>::get(
				ALICE,
				CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
			),
			TokenInfo {
				amount: 10,
				reserved: 0
			}
		);

		assert_noop!(
			Dex::withdraw(
				RuntimeOrigin::signed(ALICE),
				CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
				11
			),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::withdraw(
			RuntimeOrigin::signed(ALICE),
			CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
			10
		));
		assert_eq!(
			UserTokenInfoes::<Test>::get(
				ALICE,
				CurrencyId::Erc1155(erc1155_address(), U256::from(0)),
			),
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
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), USDT, 100));

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				USDT,
				200,
				200,
				OrderType::SELL,
				1000
			),
			Error::<Test>::PairAssetIdMustNotEqual
		);

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				GGXT,
				19, // offered
				7,  // requested
				OrderType::BUY,
				1000
			),
			// because in requested * price == offered
			// `price` cannot be an integer.
			Error::<Test>::PriceDoNotMatchOfferedRequestedAmount
		);

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				GGXT,
				19, // offered
				7,  // requested
				OrderType::SELL,
				1000
			),
			// because in requested * price == offered
			// `price` cannot be an integer.
			Error::<Test>::PriceDoNotMatchOfferedRequestedAmount
		);

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				GGXT,
				7,  // offered
				19, // requested
				OrderType::BUY,
				1000
			),
			// because in requested * price == offered
			// `price` cannot be an integer.
			Error::<Test>::PriceDoNotMatchOfferedRequestedAmount
		);

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				GGXT,
				7,  // offered
				19, // requested
				OrderType::SELL,
				1000
			),
			// because in requested * price == offered
			// `price` cannot be an integer.
			Error::<Test>::PriceDoNotMatchOfferedRequestedAmount
		);

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				GGXT,
				1,   // offered
				200, // requested
				OrderType::BUY,
				1000
			),
			// because in requested * price == offered
			// `price` cannot be an integer.
			Error::<Test>::PriceDoNotMatchOfferedRequestedAmount
		);

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
			TokenInfo {
				amount: 100,
				reserved: 0,
			}
		);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			1,
			200,
			OrderType::SELL,
			1000
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: ALICE,
				pair: (USDT, GGXT),
				expiration_block: 1000,
				amount_offered: 1,
				amout_requested: 200,
				price: 200,
				order_type: OrderType::SELL,
				unfilled_offered: 1,
				unfilled_requested: 200,
				order_status: OrderStatus::Pending,
			})
		);

		assert_eq!(UserOrders::<Test>::get(ALICE, 0), ());

		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![0]);

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
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
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), USDT, 100));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), GGXT, 200));

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
			TokenInfo {
				amount: 100,
				reserved: 0,
			}
		);

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, GGXT),
			TokenInfo {
				amount: 200,
				reserved: 0,
			}
		);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			GGXT,
			USDT,
			200,
			1,
			OrderType::SELL,
			1000
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: ALICE,
				pair: (USDT, GGXT),
				expiration_block: 1000,
				amount_offered: 200,
				amout_requested: 1,
				price: 200,
				order_type: OrderType::BUY,
				unfilled_offered: 200,
				unfilled_requested: 1,
				order_status: OrderStatus::Pending,
			})
		);

		assert_eq!(UserOrders::<Test>::get(ALICE, 0), ());

		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![0]);

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, GGXT),
			TokenInfo {
				amount: 0,
				reserved: 200,
			}
		);
	})
}

#[test]
fn test_cancel_order() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Dex::cancel_order(RuntimeOrigin::signed(ALICE), 0),
			Error::<Test>::InvalidOrderIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), USDT, 100));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			1,
			200,
			OrderType::SELL,
			1000
		));

		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: ALICE,
				pair: (USDT, GGXT),
				expiration_block: 1000,
				amount_offered: 1,
				amout_requested: 200,
				price: 200,
				order_type: OrderType::SELL,
				unfilled_offered: 1,
				unfilled_requested: 200,
				order_status: OrderStatus::Pending,
			})
		);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 0), true);
		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![0]);

		assert_noop!(
			Dex::cancel_order(RuntimeOrigin::signed(BOB), 0),
			Error::<Test>::NotOwner
		);

		assert_ok!(Dex::cancel_order(RuntimeOrigin::signed(ALICE), 0));

		assert_eq!(Orders::<Test>::get(0), None);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 1), false);
		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![]);
	})
}

#[test]
fn test_take_order_sell() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), USDT, 100));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			1,
			200,
			OrderType::SELL,
			1000
		));

		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(BOB), 0),
			Error::<Test>::UserAssetNotExist
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(BOB), GGXT, 100));
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(BOB), 0),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(BOB), GGXT, 200));

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
			TokenInfo {
				amount: 99,
				reserved: 1,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, GGXT),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, USDT),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, GGXT),
			TokenInfo {
				amount: 300,
				reserved: 0,
			}
		);

		assert_ok!(Dex::take_order(RuntimeOrigin::signed(BOB), 0));

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
			TokenInfo {
				amount: 99,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, GGXT),
			TokenInfo {
				amount: 200,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, USDT),
			TokenInfo {
				amount: 1,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, GGXT),
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
			Dex::take_order(RuntimeOrigin::signed(BOB), 0),
			Error::<Test>::InvalidOrderIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), GGXT, 200));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			200,
			200,
			OrderType::BUY,
			1000,
		));

		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(BOB), 0),
			Error::<Test>::UserAssetNotExist
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(BOB), USDT, 1));
		assert_noop!(
			Dex::take_order(RuntimeOrigin::signed(BOB), 0),
			Error::<Test>::NotEnoughBalance
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(BOB), USDT, 200));

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, GGXT),
			TokenInfo {
				amount: 0,
				reserved: 200,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, USDT),
			TokenInfo {
				amount: 201,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, GGXT),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);

		assert_ok!(Dex::take_order(RuntimeOrigin::signed(BOB), 0));

		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, USDT),
			TokenInfo {
				amount: 200,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(ALICE, GGXT),
			TokenInfo {
				amount: 0,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, USDT),
			TokenInfo {
				amount: 1,
				reserved: 0,
			}
		);
		assert_eq!(
			UserTokenInfoes::<Test>::get(BOB, GGXT),
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
			Dex::take_order(RuntimeOrigin::signed(BOB), 0),
			Error::<Test>::InvalidOrderIndex
		);

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), BTC, 200));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), GGXT, 500));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), USDT, 200));
		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			100,
			1,
			OrderType::BUY,
			1000
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			200,
			2,
			OrderType::BUY,
			1000
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			GGXT,
			BTC,
			2,
			200,
			OrderType::SELL,
			1000
		));

		assert_ok!(Dex::deposit(RuntimeOrigin::signed(BOB), BTC, 300));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(BOB), GGXT, 300));
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(BOB), USDT, 300));

		assert_eq!(Orders::<Test>::contains_key(0), true);
		assert_eq!(Orders::<Test>::contains_key(1), true);
		assert_eq!(Orders::<Test>::contains_key(2), true);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 0), true);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 1), true);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 2), true);
		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![0, 1]);
		assert_eq!(PairOrders::<Test>::get((GGXT, BTC)), vec![2]);

		assert_ok!(Dex::cancel_order(RuntimeOrigin::signed(ALICE), 1));
		assert_ok!(Dex::take_order(RuntimeOrigin::signed(BOB), 0));

		assert_eq!(Orders::<Test>::contains_key(0), false);
		assert_eq!(Orders::<Test>::contains_key(1), false);
		assert_eq!(Orders::<Test>::contains_key(2), true);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 0), false);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 1), false);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 2), true);
		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![]);
		assert_eq!(PairOrders::<Test>::get((GGXT, BTC)), vec![2]);
	})
}

#[test]
fn test_expiration_works_as_expected() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), GGXT, 200));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			100,
			1,
			OrderType::BUY,
			10
		));

		assert_eq!(Orders::<Test>::contains_key(0), true);
		assert_eq!(UserOrders::<Test>::contains_key(ALICE, 0), true);
		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![0]);
		assert_eq!(
			Orders::<Test>::get(0),
			Some(Order {
				counter: 0,
				address: ALICE,
				pair: (USDT, GGXT),
				expiration_block: 10,
				amount_offered: 100,
				amout_requested: 1,
				price: 100,
				order_type: OrderType::BUY,
				unfilled_offered: 100,
				unfilled_requested: 1,
				order_status: OrderStatus::Pending,
			}),
		);
		assert_eq!(OrderExpiration::<Test>::get(10), vec![0]);

		run_to_block(11);

		assert!(!Orders::<Test>::contains_key(0));
		assert!(!UserOrders::<Test>::contains_key(ALICE, 0));
		assert_eq!(PairOrders::<Test>::get((USDT, GGXT)), vec![]);
		assert_eq!(OrderExpiration::<Test>::get(10), vec![]);
	});
}

#[test]
fn fail_on_invalid_expiry() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::deposit(RuntimeOrigin::signed(ALICE), GGXT, 200));
		run_to_block(5);

		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				GGXT,
				100,
				1,
				OrderType::BUY,
				3
			),
			Error::<Test>::ExpirationMustBeInFuture
		);
		assert_noop!(
			Dex::make_order(
				RuntimeOrigin::signed(ALICE),
				USDT,
				GGXT,
				100,
				1,
				OrderType::BUY,
				5
			),
			Error::<Test>::ExpirationMustBeInFuture
		);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			100,
			1,
			OrderType::BUY,
			6
		));
	});
}

#[test]
fn test_offchain_worker_order_matching() {
	use frame_support::traits::OffchainWorker;
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

		@@ match order
	// price amout order_1_id  order_2_id
	[new MatchDetailRecord(bd("208760"), bd("1"), orders.get(2), orders.get(1)) ]
	[new MatchDetailRecord(bd("208760"), bd("1"), orders.get(8), orders.get(1)) ]
	[new MatchDetailRecord(bd("208654"), bd("2"), orders.get(11), orders.get(9)),  new MatchDetailRecord(bd("208655"), bd("1"), orders.get(11), orders.get(10)) ]

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

	let (offchain, _offchain_state) = TestOffchainExt::new();
	let (pool, pool_state) = TestTransactionPoolExt::new();
	ext.register_extension(OffchainDbExt::new(offchain.clone()));
	ext.register_extension(OffchainWorkerExt::new(offchain));
	ext.register_extension(TransactionPoolExt::new(pool));

	ext.execute_with(|| {
		assert_ok!(Dex::deposit(
			RuntimeOrigin::signed(ALICE),
			USDT,
			1_000_000_000
		));
		assert_ok!(Dex::deposit(
			RuntimeOrigin::signed(ALICE),
			GGXT,
			1_000_000_000
		));

		let block = 1;
		System::set_block_number(block);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			208234,
			1,
			OrderType::BUY,
			1000,
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			2,
			417520,
			OrderType::SELL,
			1000,
		));

		Dex::offchain_worker(block);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			208780,
			1,
			OrderType::BUY,
			1000,
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			1042505,
			5,
			OrderType::BUY,
			1000,
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			3,
			626406,
			OrderType::SELL,
			1000,
		));

		Dex::offchain_worker(block);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			6,
			1252560,
			OrderType::SELL,
			1000,
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			1456777,
			7,
			OrderType::BUY,
			1000,
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			625800,
			3,
			OrderType::BUY,
			1000,
		));
		Dex::offchain_worker(block);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			208833,
			1,
			OrderType::BUY,
			1000,
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			2,
			417308,
			OrderType::SELL,
			1000,
		));

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			5,
			1043275,
			OrderType::SELL,
			1000,
		));
		Dex::offchain_worker(block);

		assert_ok!(Dex::make_order(
			RuntimeOrigin::signed(ALICE),
			USDT,
			GGXT,
			625965,
			3,
			OrderType::BUY,
			1000,
		));

		Dex::offchain_worker(block);

		let mut txs = vec![];
		while !pool_state.read().transactions.is_empty() {
			let tx = pool_state.write().transactions.pop().unwrap();
			let tx = Extrinsic::decode(&mut &*tx).unwrap();
			txs.insert(0, tx);
		}

		for tx in txs {
			match tx.call {
				RuntimeCall::Dex(crate::Call::update_match_order_unsigned { match_result: m }) => {
					let _ = Dex::update_match_order_unsigned(RuntimeOrigin::none(), m);
				}
				_ => {
					assert_eq!(2, 3);
				}
			};
		}

		//order_book  price=> (total_offered_amount, total_requested_amount)
		let mut sell_order_book = BTreeMap::new();
		let mut buy_order_book = BTreeMap::new();

		for (_, order) in Orders::<Test>::iter() {
			if order.order_status != OrderStatus::FullyFilled {
				if order.order_type == OrderType::BUY {
					if !buy_order_book.contains_key(&order.price) {
						buy_order_book.insert(
							order.price,
							(order.unfilled_offered, order.unfilled_requested),
						);
					} else {
						let v = buy_order_book.get(&order.price).unwrap();

						buy_order_book.insert(
							order.price,
							(v.0 + order.unfilled_offered, v.1 + order.unfilled_requested),
						);
					}
				} else {
					if !sell_order_book.contains_key(&order.price) {
						sell_order_book.insert(
							order.price,
							(order.unfilled_offered, order.unfilled_requested),
						);
					} else {
						let v = sell_order_book.get(&order.price).unwrap();
						sell_order_book.insert(
							order.price,
							(v.0 + order.unfilled_offered, v.1 + order.unfilled_requested),
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
