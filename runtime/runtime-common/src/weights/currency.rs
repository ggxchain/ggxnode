#![allow(unused_parens, unused_imports, clippy::unnecessary_cast)]
use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

// The weight info trait for `currency`.
pub trait WeightInfo {
	fn change_inflation_percent() -> Weight;
	fn change_inflation_decay() -> Weight;
	fn yearly_inflation_decay() -> Weight;
	fn change_treasury_commission() -> Weight;
	fn change_treasury_commission_from_fee() -> Weight;
	fn change_treasury_commission_from_tips() -> Weight;
}

/// Weights for currency using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn change_inflation_percent() -> Weight {
		Weight::from_all(100_000_u64)
	}
	fn change_inflation_decay() -> Weight {
		Weight::from_all(100_000_u64)
	}
	fn yearly_inflation_decay() -> Weight {
		Weight::from_all(100_000_u64)
	}
	fn change_treasury_commission() -> Weight {
		Weight::from_all(100_000_u64)
	}
	fn change_treasury_commission_from_fee() -> Weight {
		Weight::from_all(100_000_u64)
	}
	fn change_treasury_commission_from_tips() -> Weight {
		Weight::from_all(100_000_u64)
	}
}
