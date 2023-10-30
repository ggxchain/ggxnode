#![allow(unused_parens, unused_imports, clippy::unnecessary_cast)]
use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

// The weight info trait for `validator_manager`.
pub trait WeightInfo {
	fn register_validators(i: u32) -> Weight;
	fn deregister_validators(i: u32) -> Weight;
}

/// Weights for `validator_manager` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn register_validators(_i: u32) -> Weight {
		Weight::from_all(100_000_u64)
	}
	fn deregister_validators(_i: u32) -> Weight {
		Weight::from_all(100_000_u64)
	}
}
