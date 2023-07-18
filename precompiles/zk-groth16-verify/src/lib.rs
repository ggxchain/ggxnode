#![cfg_attr(not(feature = "std"), no_std)]

use pallet_evm::{ExitError, ExitSucceed, LinearCostPrecompile, PrecompileFailure};

use core::ops::Range;
use sp_std::{prelude::Box, vec::Vec};

use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;

use ark::{ark_bn254_fr, ark_bn254_g1, ark_bn254_g2};
use ethabi::Token;

pub struct ZKGroth16Verify;

mod ark;

impl LinearCostPrecompile for ZKGroth16Verify {
	const BASE: u64 = 60;
	const WORD: u64 = 12;

	fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
		log::trace!(target: "zk-groth16-verify-precompile", "In zk-groth16-verify");
		const MIN_INPUT_LENGTH: usize = 24 * 32;
		if input.len() < 4 + MIN_INPUT_LENGTH {
			return Err(PrecompileFailure::from(ExitError::InvalidRange));
		}
		// let input_stripped = &input[4..];
		let (_selector, input_stripped) = input.split_at(4);

		let mut i256_r = Int256Reader::new();
		let proof_a_x = &input_stripped[i256_r.next()];
		let proof_a_y = &input_stripped[i256_r.next()];
		let proof_b_x1 = &input_stripped[i256_r.next()];
		let proof_b_x2 = &input_stripped[i256_r.next()];
		let proof_b_y1 = &input_stripped[i256_r.next()];
		let proof_b_y2 = &input_stripped[i256_r.next()];
		let proof_c_x = &input_stripped[i256_r.next()];
		let proof_c_y = &input_stripped[i256_r.next()];

		let vk_alpha_x = &input_stripped[i256_r.next()];
		let vk_alpha_y = &input_stripped[i256_r.next()];
		let vk_beta_x1 = &input_stripped[i256_r.next()];
		let vk_beta_x2 = &input_stripped[i256_r.next()];
		let vk_beta_y1 = &input_stripped[i256_r.next()];
		let vk_beta_y2 = &input_stripped[i256_r.next()];
		let vk_gamma_x1 = &input_stripped[i256_r.next()];
		let vk_gamma_x2 = &input_stripped[i256_r.next()];
		let vk_gamma_y1 = &input_stripped[i256_r.next()];
		let vk_gamma_y2 = &input_stripped[i256_r.next()];
		let vk_delta_x1 = &input_stripped[i256_r.next()];
		let vk_delta_x2 = &input_stripped[i256_r.next()];
		let vk_delta_y1 = &input_stripped[i256_r.next()];
		let vk_delta_y2 = &input_stripped[i256_r.next()];

		// It seems like whole data should be decoded to decode arrays.
		let types = &[
			ethabi::ParamType::Uint(256), // proof a x
			ethabi::ParamType::Uint(256), // proof a y
			ethabi::ParamType::Uint(256), // proof b x1
			ethabi::ParamType::Uint(256), // proof b x2
			ethabi::ParamType::Uint(256), // proof b y1
			ethabi::ParamType::Uint(256), // proof b y2
			ethabi::ParamType::Uint(256), // proof c x
			ethabi::ParamType::Uint(256), // proof c y
			ethabi::ParamType::Uint(256), // vk alpha x
			ethabi::ParamType::Uint(256), // vk alpha y
			ethabi::ParamType::Uint(256), // vk beta x1
			ethabi::ParamType::Uint(256), // vk beta x2
			ethabi::ParamType::Uint(256), // vk beta y1
			ethabi::ParamType::Uint(256), // vk beta y2
			ethabi::ParamType::Uint(256), // vk gamma x1
			ethabi::ParamType::Uint(256), // vk gamma x2
			ethabi::ParamType::Uint(256), // vk gamma y1
			ethabi::ParamType::Uint(256), // vk gamma y2
			ethabi::ParamType::Uint(256), // vk delta x1
			ethabi::ParamType::Uint(256), // vk delta x2
			ethabi::ParamType::Uint(256), // vk delta y1
			ethabi::ParamType::Uint(256), // vk delta y2
			ethabi::ParamType::Array(Box::from(ethabi::ParamType::FixedArray(
				Box::from(ethabi::ParamType::Uint(256)),
				2,
			))), // vk ic
			ethabi::ParamType::Array(Box::from(ethabi::ParamType::Uint(256))), // input
		];
		let decoded = ethabi::decode(types, input_stripped)
			.map_err(|_e| PrecompileFailure::from(ExitError::InvalidRange))?;

		let pub_arr = decoded[decoded.len() - 1]
			.clone()
			.into_array()
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let pub_inputs: Vec<_> = pub_arr
			.iter()
			.map(|t| ethabi::encode(&[t.clone()]))
			.map(|b| ark_bn254_fr(&b))
			.collect();
		let proof_a = ark_bn254_g1(proof_a_x, proof_a_y)
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let proof_b = ark_bn254_g2(proof_b_x1, proof_b_x2, proof_b_y1, proof_b_y2)
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let proof_c = ark_bn254_g1(proof_c_x, proof_c_y)
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let proof: ark_groth16::Proof<ark_ec::bn::Bn<ark_bn254::Config>> = ark_groth16::Proof {
			a: proof_a,
			b: proof_b,
			c: proof_c,
		};

		let ic_arr = decoded[decoded.len() - 2]
			.clone()
			.into_array()
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;

		let vk_ic: Vec<_> = ic_arr
			.iter()
			.map(|point| {
				let array = point
					.clone()
					.into_fixed_array()
					.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
				let t1 = array[0].clone();
				let t2 = array[1].clone();
				let x = ethabi::encode(&[t1]);
				let y = ethabi::encode(&[t2]);
				ark_bn254_g1(&x, &y).ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))
			})
			.collect::<Result<Vec<_>, PrecompileFailure>>()?;

		let vk_alpha_g1 = ark_bn254_g1(vk_alpha_x, vk_alpha_y)
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let vk_beta_g2 = ark_bn254_g2(vk_beta_x1, vk_beta_x2, vk_beta_y1, vk_beta_y2)
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let vk_gamma_g2 = ark_bn254_g2(vk_gamma_x1, vk_gamma_x2, vk_gamma_y1, vk_gamma_y2)
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;
		let vk_delta_g2 = ark_bn254_g2(vk_delta_x1, vk_delta_x2, vk_delta_y1, vk_delta_y2)
			.ok_or_else(|| PrecompileFailure::from(ExitError::InvalidRange))?;

		let vk: ark_groth16::VerifyingKey<ark_ec::bn::Bn<ark_bn254::Config>> =
			ark_groth16::VerifyingKey {
				alpha_g1: vk_alpha_g1,
				beta_g2: vk_beta_g2,
				gamma_g2: vk_gamma_g2,
				delta_g2: vk_delta_g2,
				gamma_abc_g1: vk_ic,
			};

		let verified = Groth16::<ark_bn254::Bn254>::verify(&vk, &pub_inputs, &proof)
			.map_err(|_e| PrecompileFailure::from(ExitError::InvalidRange))?;
		log::debug!(
			target: "runtime_common::precompiles::zk_verify::execute",
			"verification result {:?}",
			verified
		);

		let encoded = ethabi::encode(&[Token::Bool(verified)]);
		Ok((ExitSucceed::Returned, encoded.to_vec()))
	}
}

struct Int256Reader {
	offset: usize,
}

impl Int256Reader {
	fn new() -> Self {
		Int256Reader { offset: 0 }
	}

	fn next(&mut self) -> Range<usize> {
		let prev = self.offset;
		self.offset += 32;

		prev..self.offset
	}
}
