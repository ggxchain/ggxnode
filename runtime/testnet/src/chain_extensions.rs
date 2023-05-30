use super::Runtime;

use frame_support::log::error;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::DispatchError;

/// Contract extension for `IBCISC20Extension`
#[derive(Default)]
pub struct IBCISC20Extension;

impl ChainExtension<Runtime> for IBCISC20Extension {
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		let func_id = env.func_id();
		match func_id {
			1 => {}
			_ => {
				error!("Called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("Unimplemented func_id"));
			}
		}
		Ok(RetVal::Converging(0))
	}
}
