use super::{prelude::*, Runtime, RuntimeEvent};

parameter_types! {
	pub const EvmId: u8 = 0x0F;
	pub const WasmId: u8 = 0x1F;
}

use pallet_xvm::{evm, wasm};
impl pallet_xvm::Config for Runtime {
	type SyncVM = (evm::EVM<EvmId, Self>, wasm::WASM<WasmId, Self>);
	type AsyncVM = ();
	type RuntimeEvent = RuntimeEvent;
}
