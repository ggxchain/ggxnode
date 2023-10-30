#![allow(unused_parens, unused_imports, clippy::unnecessary_cast)]
use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

// The weight info trait for `session_payout`.
pub trait WeightInfo {
	fn change_validator_to_nominator_commission_algorithm() -> Weight;
}

/// Weights for `session_payout` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn change_validator_to_nominator_commission_algorithm() -> Weight {
		Weight::from_all(100_000_u64)
	}
}
