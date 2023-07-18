use pallet_evm::{
	IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use sp_core::H160;
use sp_std::marker::PhantomData;

use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_ed25519::Ed25519Verify;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_session::SessionWrapper;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompile_sr25519::Sr25519Precompile;
use pallet_evm_precompile_substrate_ecdsa::SubstrateEcdsaPrecompile;
use pallet_evm_precompile_xvm::XvmPrecompile;
use pallet_evm_precompile_zk_groth16_verify::ZKGroth16Verify;

#[derive(Default)]
pub struct GoldenGatePrecompiles<R>(PhantomData<R>);

pub mod consts {
	use sp_core::H160;

	pub const EC_RECOVER: H160 = hash(1);
	pub const SHA256: H160 = hash(2);
	pub const RIPEMD160: H160 = hash(3);
	pub const IDENTITY: H160 = hash(4);
	pub const MODEXP: H160 = hash(5);
	pub const BN128_ADD: H160 = hash(6);
	pub const BN128_MUL: H160 = hash(7);
	pub const BN128_PAIRING: H160 = hash(8);
	pub const BLAKE2F: H160 = hash(9);

	/// F3 is also used in Celo, so preserve the address for contracts interoperability.
	pub const ED25519_VERIFY_CELO: H160 = hash(0xF3);
	pub const SHA3_FIPS256: H160 = hash(0x400);
	/// 402 is used in Astar and Moonbeam, so preserve the address for contracts interoperability
	pub const EC_RECOVER_PUBLIC_KEY: H160 = hash(0x402);
	/// 403 is used in Astar, so preserve the address for contracts interoperability
	pub const ED25519_VERIFY_ASTAR: H160 = hash(0x403);
	/// 5002 is used in Astar, so preserve the address for contracts interoperability
	pub const SR25519_VERIFY: H160 = hash(0x5002);
	/// 5003 is used in Astar, so preserve the address for contracts interoperability
	pub const ECDSA_VERIFY: H160 = hash(0x5003);
	/// 5005 is used in Astar, so preserve the address for contracts interoperability
	pub const XVM: H160 = hash(0x5005);

	pub const SESSION_WRAPPER: H160 = hash(0x2052);

	pub const ZK_GROTH16_VERIFY: H160 = hash(0x8888);
	//
	pub const SUPPORTED_PRECOMPILES: [H160; 18] = [
		EC_RECOVER,
		SHA256,
		RIPEMD160,
		IDENTITY,
		MODEXP,
		BN128_ADD,
		BN128_MUL,
		BN128_PAIRING,
		BLAKE2F,
		ED25519_VERIFY_CELO,
		SHA3_FIPS256,
		EC_RECOVER_PUBLIC_KEY,
		ED25519_VERIFY_ASTAR,
		SR25519_VERIFY,
		ECDSA_VERIFY,
		XVM,
		SESSION_WRAPPER,
		ZK_GROTH16_VERIFY,
	];

	const fn hash(a: u64) -> H160 {
		let bytes = a.to_be_bytes();
		let mut result = [0u8; 20];
		result[12] = bytes[0];
		result[13] = bytes[1];
		result[14] = bytes[2];
		result[15] = bytes[3];
		result[16] = bytes[4];
		result[17] = bytes[5];
		result[18] = bytes[6];
		result[19] = bytes[7];
		H160(result)
	}

	#[cfg(test)]
	mod tests {
		use super::hash;
		use sp_core::H160;
		use test_strategy::proptest;

		#[proptest]
		fn hash_function_is_correct(a: u64) {
			assert_eq!(hash(a), H160::from_low_u64_be(a));
		}
	}
}

impl<R> GoldenGatePrecompiles<R> {
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
	/// * 0x6 - is EcAdd
	/// * 0x7 - is EcMul
	/// * 0x8 - is EcPairing
	/// * 0x9 - is Blake2F
	///
	/// The next list contains handy precompiles that are missing in Ethereum.
	/// Please note we use 0xF3 and 0x403 for `Ed25519 verify` to be compatible with Celo and Astar
	/// * 0xF3 - is Ed25519 verify
	/// * 0x400 - is Sha3
	/// * 0x402 - is ECRecoverPublicKey (402 is used in Astar and Moonbeam, so preserve the address for contracts interoperability)
	/// * 0x403 - is Ed25519 verify (403 is used in Astar, so preserve the address for contracts interoperability)
	///
	/// The next list contains Astar specific precompiles:
	/// * 0x5002 - is Sr25519 verify
	/// * 0x5003 - is Ecdsa verify
	/// * 0x5005 - is cross virtual machine (XVM)
	pub fn used_addresses() -> impl Iterator<Item = H160> {
		consts::SUPPORTED_PRECOMPILES.into_iter()
	}
}

impl<R> PrecompileSet for GoldenGatePrecompiles<R>
where
	XvmPrecompile<R>: Precompile,
	SessionWrapper<R>: Precompile,
	R: pallet_evm::Config + pallet_xvm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		match handle.code_address() {
			// Ethereum precompiles :
			a if a == consts::EC_RECOVER => Some(ECRecover::execute(handle)),
			a if a == consts::SHA256 => Some(Sha256::execute(handle)),
			a if a == consts::RIPEMD160 => Some(Ripemd160::execute(handle)),
			a if a == consts::IDENTITY => Some(Identity::execute(handle)),
			a if a == consts::MODEXP => Some(Modexp::execute(handle)),
			a if a == consts::BN128_ADD => Some(Bn128Add::execute(handle)),
			a if a == consts::BN128_MUL => Some(Bn128Mul::execute(handle)),
			a if a == consts::BN128_PAIRING => Some(Bn128Pairing::execute(handle)),
			a if a == consts::BLAKE2F => Some(Blake2F::execute(handle)),
			// nor Ethereum precompiles :
			a if a == consts::ED25519_VERIFY_CELO || a == consts::ED25519_VERIFY_ASTAR => {
				Some(Ed25519Verify::execute(handle))
			}
			a if a == consts::SHA3_FIPS256 => Some(Sha3FIPS256::execute(handle)),
			a if a == consts::EC_RECOVER_PUBLIC_KEY => Some(ECRecoverPublicKey::execute(handle)),
			// Astar precompiles:
			a if a == consts::SR25519_VERIFY => Some(Sr25519Precompile::<R>::execute(handle)),
			a if a == consts::ECDSA_VERIFY => Some(SubstrateEcdsaPrecompile::<R>::execute(handle)),
			// 0x5005 - is cross virtual machine (XVM)
			a if a == consts::XVM => Some(XvmPrecompile::<R>::execute(handle)),
			a if a == consts::SESSION_WRAPPER => Some(SessionWrapper::<R>::execute(handle)),

			// 0x8888 zk
			a if a == consts::ZK_GROTH16_VERIFY => Some(ZKGroth16Verify::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
		IsPrecompileResult::Answer {
			is_precompile: Self::used_addresses().any(|x| x == address),
			extra_cost: 0,
		}
	}
}
