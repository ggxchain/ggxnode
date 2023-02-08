use pallet_evm::{Precompile, PrecompileHandle, PrecompileResult, PrecompileSet};
use sp_core::H160;
use sp_std::marker::PhantomData;

use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompile_xvm::XvmPrecompile;

pub struct FrontierPrecompiles<R>(PhantomData<R>);

impl<R> FrontierPrecompiles<R> {
	pub fn new() -> Self {
		Self(Default::default())
	}

	/// The function contains list of supported Ethereum precompiles.
	/// The next list contains Ethereum related precompiles:
	/// * 0x1 - is ECRecover
	/// * 0x2 - is Sha256
	/// * 0x3 - is Ripemd160
	/// * 0x4 - is Identity
	/// * 0x5 - is Modexp
	///
	/// The next list contains handy precompiles that are missing in Ethereum
	/// * 0x400 - is Sha3FIPS256
	/// * 0x401 - is ECRecoverPublicKey
	/// * 0x5005 - is cross virtual machine (XVM)
	pub fn used_addresses() -> impl Iterator<Item = H160> {
		[1, 2, 3, 4, 5, 1024, 1025, 20485].into_iter().map(hash)
	}
}

impl<R> PrecompileSet for FrontierPrecompiles<R>
where
	XvmPrecompile<R>: Precompile,
	R: pallet_evm::Config + pallet_xvm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		match handle.code_address() {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(handle)),
			a if a == hash(2) => Some(Sha256::execute(handle)),
			a if a == hash(3) => Some(Ripemd160::execute(handle)),
			a if a == hash(4) => Some(Identity::execute(handle)),
			a if a == hash(5) => Some(Modexp::execute(handle)),
			// Non-Frontier specific nor Ethereum precompiles :
			a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
			a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
			// Xvm 0x5005
			a if a == hash(20485) => Some(XvmPrecompile::<R>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		Self::used_addresses().any(|h| h == address)
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
