use super::Runtime;
/// Registered WASM contracts chain extensions.
///
use pallet_contracts::chain_extension::RegisteredChainExtension;

pub use pallet_chain_extension_xvm::XvmExtension;

impl RegisteredChainExtension<Runtime> for XvmExtension<Runtime> {
    const ID: u16 = 01;
}