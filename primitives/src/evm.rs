use astar_primitives::xvm::Context;

use frame_support::inherent::Vec;
use sp_runtime::{DispatchError, DispatchResult};

pub type EvmAddress = sp_core::H160;

/// An abstraction of EVMBridge
pub trait EVMBridgeTrait<AccountId, Balance> {
	/// Execute ERC20.name() to read token name from ERC20 contract
	fn name(context: Context) -> Result<Vec<u8>, DispatchError>;
	/// Execute ERC20.symbol() to read token symbol from ERC20 contract
	fn symbol(context: Context) -> Result<Vec<u8>, DispatchError>;
	/// Execute ERC20.decimals() to read token decimals from ERC20 contract
	fn decimals(context: Context) -> Result<u8, DispatchError>;
	/// Execute ERC20.totalSupply() to read total supply from ERC20 contract
	fn total_supply(context: Context) -> Result<Balance, DispatchError>;
	/// Execute ERC20.balanceOf(address) to read balance of address from ERC20
	/// contract
	fn balance_of(
		context: Context,
		contract: H160,
		from: AccountIdOf<T>,
		address: EvmAddress,
	) -> Result<Balance, DispatchError>;
	/// Execute ERC20.transfer(address, uint256) to transfer value to `to`
	fn transfer(
		context: Context,
		contract: EvmAddress,
		from: AccountId,
		to: EvmAddress,
		value: Balance,
	) -> DispatchResult;
	// /// Get the real origin account and charge storage rent from the origin.
	// fn get_origin() -> Option<AccountId>;
	// /// Set the EVM origin
	// fn set_origin(origin: AccountId);
	// /// Kill the EVM origin
	// fn kill_origin();
	// /// Push new EVM origin in xcm
	// fn push_xcm_origin(origin: AccountId);
	// /// Pop EVM origin in xcm
	// fn pop_xcm_origin();
	// /// Kill the EVM origin in xcm
	// fn kill_xcm_origin();
	// /// Get the real origin account or xcm origin and charge storage rent from the origin.
	// fn get_real_or_xcm_origin() -> Option<AccountId>;
}
