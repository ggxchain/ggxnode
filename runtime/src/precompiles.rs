use pallet_evm::{Precompile, PrecompileHandle, PrecompileResult, PrecompileSet};
use sp_core::H160;
use sp_std::marker::PhantomData;

use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompile_sr25519::Sr25519Precompile;
use pallet_evm_precompile_substrate_ecdsa::SubstrateEcdsaPrecompile;
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
	/// * 0x6 - is Alt_bn128_Add
	/// * 0x7 - is Alt_bn128_Mul
	/// * 0x8 - is Alt_bn128_pairing
	/// * 0x9 - is Blake2F
	///
	/// The next list contains handy precompiles that are missing in Ethereum
	/// * 0x400 - is standart Sha3 precompile
	/// * 0x402 - is ECRecoverPublicKey (402 is used in Astar and Moonbeam, so preserve the address for contracts interoperability)
	/// * 0x403 - is Ed25519Verify (403 is also used in Astar, so preserve the address for contracts interoperability)
	///
	/// The next list contains Astar specific precompiles:
	/// * 0x5002 - is Sr25519 verify
	/// * 0x5003 - is Ecdsa verify
	/// * 0x5005 - is cross virtual machine (XVM)
	pub fn used_addresses() -> impl Iterator<Item = H160> {
		[
			1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1026, 1027, 20482, 20483, 20485,
		]
		.into_iter()
		.map(hash)
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
			a if a == hash(6) => Some(Bn128Add::execute(handle)),
			a if a == hash(7) => Some(Bn128Mul::execute(handle)),
			a if a == hash(8) => Some(Bn128Pairing::execute(handle)),
			a if a == hash(9) => Some(Blake2F::execute(handle)),
			// nor Ethereum precompiles :
			a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
			// TODO: there is also 1025 dispatch call, but it is not used in moonbeam and it's not clear what it is
			a if a == hash(1026) => Some(ECRecoverPublicKey::execute(handle)),
			a if a == hash(1027) => Some(Ed25519Verify::execute(handle)),
			// Astar precompiles:

			// Sr25519 0x5002
			a if a == hash(20482) => Some(Sr25519Precompile::<R>::execute(handle)),
			// SubstrateEcdsa 0x5003
			a if a == hash(20483) => Some(SubstrateEcdsaPrecompile::<R>::execute(handle)),
			// 0x5005 - is cross virtual machine (XVM)
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
