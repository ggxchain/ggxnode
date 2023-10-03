#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileOutput;
use frame_support::inherent::Vec;
use pallet_evm::{Precompile, PrecompileHandle};
use precompile_utils::{
	revert, succeed, Address, EvmDataWriter, EvmResult, FunctionModifier, PrecompileHandleExt,
};
use sp_core::H256;
use sp_std::{fmt::Debug, marker::PhantomData};

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	LogsForReceipt = "logs_for_receipt(uint256,uint256,bytes32,address)",
}

/// A precompile to wrap the functionality from chain
pub struct EthReceiptPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for EthReceiptPrecompile<Runtime>
where
	Runtime: pallet_receipt_registry::Config + pallet_evm::Config + frame_system::Config,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::error!(target: "eth-receipt-provider-precompile", "In eth-receipt-provider wrapper");

		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::LogsForReceipt => FunctionModifier::NonPayable,
		})?;

		match selector {
			// Dispatchables
			Action::LogsForReceipt => Self::logs_for_receipt(handle),
		}
	}
}

impl<Runtime> EthReceiptPrecompile<Runtime>
where
	Runtime: pallet_receipt_registry::Config + pallet_evm::Config + frame_system::Config,
{
	// The dispatchable wrappers are next. They dispatch a Substrate inner Call.
	fn logs_for_receipt(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		const TARGET: &str = "eth-receipt-provider-precompile";

		let mut input = handle.read_input()?;
		input.expect_arguments(4)?;

		let chain_id = input.read::<u32>()?;
		let block_number = input.read::<u64>()?;
		let receipt_hash = input.read::<H256>()?;
		let contract_address = input.read::<Address>()?;

		log::error!(
			target:TARGET,
			"logs_for_receipt with receipt hash: {receipt_hash:?} and contract address: {contract_address:?}",
		);

		let contract_address = eth_light_client_types::H160(contract_address.0 .0);

		let (topics, data): (Vec<_>, Vec<_>) =
			pallet_receipt_registry::Pallet::<Runtime>::processed_receipts((
				webb_proposals::TypedChainId::Evm(chain_id),
				block_number,
				eth_light_client_types::H256(receipt_hash.0),
			))
			.ok_or(revert("receipt not found"))?
			.into_iter()
			.filter(|log| log.address == contract_address)
			.map(|log| {
				let topics = log
					.topics
					.into_iter()
					.map(|topic| sp_core::H256(topic.0))
					.collect::<Vec<_>>();
				(topics, log.data)
			})
			.unzip();
		Ok(succeed(
			EvmDataWriter::new().write(topics).write(data).build(),
		))
	}
}
