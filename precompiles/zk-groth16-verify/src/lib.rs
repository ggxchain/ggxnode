#![cfg_attr(not(feature = "std"), no_std)]

use pallet_evm::{ExitError, ExitSucceed, LinearCostPrecompile, PrecompileFailure};
use sp_core::U256;

use sp_std::vec::Vec;

use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;

use ark::{ark_bn254_fr, ark_bn254_g1, ark_bn254_g2};
use ark_std::format;
use precompile_utils::{EvmDataReader, EvmDataWriter};

pub struct ZKGroth16Verify;

mod ark;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	Verify = "verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])",
}

impl LinearCostPrecompile for ZKGroth16Verify {
	const BASE: u64 = 60;
	const WORD: u64 = 12;

	fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
		log::trace!(target: "precompiles::zk_groth16_verify::execute", "In zk-groth16-verify");
		const MIN_INPUT_LENGTH: usize = 24 * 32;
		let selector: Action = EvmDataReader::read_selector(input)?;

		if input.len() < 4 + MIN_INPUT_LENGTH {
			return Err(PrecompileFailure::from(ExitError::InvalidRange));
		}
		match selector {
			Action::Verify => Self::verify(&input[4..]),
		}
	}
}

impl ZKGroth16Verify {
	pub fn verify(input_stripped: &[u8]) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
		let mut cursor = 0;
		let mut next = || {
			let start = cursor;
			cursor += 32;
			&input_stripped[start..cursor]
		};

		let proof_a = ark_bn254_g1(next(), next())
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let proof_b = ark_bn254_g2(next(), next(), next(), next())
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let proof_c = ark_bn254_g1(next(), next())
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;

		let proof: ark_groth16::Proof<ark_ec::bn::Bn<ark_bn254::Config>> = ark_groth16::Proof {
			a: proof_a,
			b: proof_b,
			c: proof_c,
		};
		log::debug!(
			target: "precompiles::zk_groth16_verify::execute",
			"Proof: {:?}",
			proof
		);

		let vk_alpha = ark_bn254_g1(next(), next())
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let vk_beta = ark_bn254_g2(next(), next(), next(), next())
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let vk_gamma = ark_bn254_g2(next(), next(), next(), next())
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let vk_delta = ark_bn254_g2(next(), next(), next(), next())
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;

		// Read the offset of vk_ic and skip the length field
		let vk_ic_offset = U256::from_big_endian(next()).low_u32() as usize + 32;

		// Read the offset of input
		let mut input_offset = U256::from_big_endian(next()).low_u32() as usize;
		// Read the length of the input array
		let input_len = U256::from_big_endian(&input_stripped[input_offset..input_offset + 32])
			.low_u32() as usize;
		// Skip the length field
		input_offset += 32;

		// Read the vk_ic array
		let vk_ic = input_stripped[vk_ic_offset..input_offset]
			.chunks_exact(64)
			.map(|chunk| {
				let (a, b) = chunk.split_at(32);
				ark_bn254_g1(a, b).ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))
			})
			.collect::<Result<Vec<_>, _>>()?;

		// Read the input array
		let pub_inputs: Vec<ark_bn254::Fr> = input_stripped[input_offset..]
			.chunks_exact(32)
			.map(ark_bn254_fr)
			.collect();
		log::info!(
			target: "precompiles::zk_groth16_verify::execute",
			"Pub_inputs: {:?}",
			pub_inputs
		);

		let mut bigints: Vec<num_bigint::BigUint> = input_stripped[input_offset..]
			.chunks(32)
			.map(|chunk| {
				let chunk = &chunk[24..32];
				num_bigint::BigUint::from_bytes_be(chunk)
			})
			.collect();
		bigints.resize(4, num_bigint::BigUint::from(0u64));
let bigints_bytes: Vec<u8> = bigints
    .iter()
    .flat_map(|b| {
        let mut bytes = vec![0u8; 8];
        let b_bytes = b.to_bytes_be();
        bytes[(8 - b_bytes.len())..].copy_from_slice(&b_bytes);
        bytes
    })
    .collect();
		log::info!(
			target: "precompiles::zk_groth16_verify::execute",
			"bigints: {:?}",
			bigints_bytes
		);
		log::info!(
			target: "precompiles::zk_groth16_verify::execute",
			"bigints_len: {:?}",
			bigints
			.iter()
			.flat_map(|b| b.to_bytes_be())
			.collect::<Vec<u8>>()
		);

		let pub_inputs: Vec<ark_bn254::Fr> = bigints
			.iter()
			.flat_map(|b| b.to_bytes_be())
			.collect::<Vec<u8>>()
			.chunks_exact(32)
			.map(ark_bn254_fr)
			.collect();
		// let pub_inputs: Vec<ark_ff::Fp<ark_ff::MontBackend<ark_bn254::FrConfig, 4>, 4>> =
		// 	vec![ark_bn254_fr(&bigints_bytes)];

		log::info!(
			target: "precompiles::zk_groth16_verify::execute",
			"Pub_inputs: {:?}",
			pub_inputs
		);
		let vk: ark_groth16::VerifyingKey<ark_ec::bn::Bn<ark_bn254::Config>> =
			ark_groth16::VerifyingKey {
				alpha_g1: vk_alpha,
				beta_g2: vk_beta,
				gamma_g2: vk_gamma,
				delta_g2: vk_delta,
				gamma_abc_g1: vk_ic,
			};
		log::debug!(
			target: "precompiles::zk_groth16_verify::execute",
			"VerifyingKey: {:?}",
			vk
		);

		let verified =
			Groth16::<ark_bn254::Bn254>::verify(&vk, &pub_inputs, &proof).map_err(|e| {
				let error_message = format!("{e}"); // Convert the error to a string
				PrecompileFailure::Error {
					exit_status: ExitError::Other(error_message.into()),
				}
			})?;

		log::debug!(
			target: "precompiles::zk_groth16_verify::execute",
			"Verification result {:?}",
			verified
		);

		Ok((
			ExitSucceed::Returned,
			EvmDataWriter::new().write(verified).build(),
		))
	}
}
