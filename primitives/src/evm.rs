use astar_primitives::xvm::Context;

use frame_support::inherent::Vec;
use sp_core::U256;
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
		contract: EvmAddress,
		caller: AccountId,
		address: EvmAddress,
	) -> Result<Balance, DispatchError>;
	/// Execute ERC20.transfer(address, uint256) to transfer value to `to`
	fn transfer(
		context: Context,
		contract: EvmAddress,
		caller: AccountId,
		to: EvmAddress,
		value: Balance,
	) -> DispatchResult;
}

/// An abstraction of EVMBridge
pub trait EVMERC1155BridgeTrait<AccountId, Balance> {
	/// Execute ERC1155.balanceOf(address _owner, uint256 _id) to read balance of address from ERC20
	/// contract
	fn balance_of(
		context: Context,
		contract: EvmAddress,
		caller: AccountId,
		address: EvmAddress,
		id: U256,
	) -> Result<Balance, DispatchError>;

	/// Execute ERC1155.safeTransferFrom(address _from, address _to, uint256 _id, uint256 _value, bytes calldata _data) to transfer value to `to`
	#[allow(clippy::too_many_arguments)]
	fn safe_transfer_from(
		context: Context,
		contract: EvmAddress,
		caller: AccountId,
		from: EvmAddress,
		to: EvmAddress,
		id: U256,
		value: Balance,
		data: Vec<u8>,
	) -> DispatchResult;
}
