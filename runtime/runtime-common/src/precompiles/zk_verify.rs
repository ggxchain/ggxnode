use pallet_evm::{ExitError, ExitSucceed, LinearCostPrecompile, PrecompileFailure};

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use core::ops::Range;

use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;

use crate::precompiles::zk_verify::ark::{ark_bn254_fr, ark_bn254_proof, ark_bn254_vk};
use ethabi::Token;

pub struct ZKGroth16Verify;

mod ark;

impl LinearCostPrecompile for ZKGroth16Verify {
	const BASE: u64 = 60;
	const WORD: u64 = 12;

	fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
		let input_stripped = &input[4..];
		let mut i256_r = Int256Reader::new();

		if input_stripped.len() < 704 {
			return Err(PrecompileFailure::from(ExitError::InvalidRange));
		}

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
		let decoded = match ethabi::decode(types, input_stripped) {
			Ok(v) => v,
			Err(_e) => {
				return Err(PrecompileFailure::from(ExitError::InvalidRange));
			}
		};

		let pub_arr = match decoded[decoded.len() - 1].clone().into_array() {
			Some(v) => v,
			None => return Err(PrecompileFailure::from(ExitError::InvalidRange)),
		};
		let pub_inputs: Vec<_> = pub_arr
			.iter()
			.map(|t| ethabi::encode(&[t.clone()]))
			.map(|b| ark_bn254_fr(&b))
			.collect();

		let proof = ark_bn254_proof(
			proof_a_x, proof_a_y, proof_b_x1, proof_b_x2, proof_b_y1, proof_b_y2, proof_c_x,
			proof_c_y,
		);

		let mut vk_ic_: Vec<[Vec<u8>; 2]> = Vec::new();

		let ic_arr = match decoded[decoded.len() - 2].clone().into_array() {
			Some(v) => v,
			None => return Err(PrecompileFailure::from(ExitError::InvalidRange)),
		};
		for point in ic_arr {
			let array = match point.clone().into_fixed_array() {
				Some(v) => v,
				None => return Err(PrecompileFailure::from(ExitError::InvalidRange)),
			};

			let t1 = array[0].clone();
			let t2 = array[1].clone();

			let x = ethabi::encode(&[t1]);
			let y = ethabi::encode(&[t2]);

			vk_ic_.push([x, y]);
		}

		let vk = ark_bn254_vk(
			vk_alpha_x,
			vk_alpha_y,
			vk_beta_x1,
			vk_beta_x2,
			vk_beta_y1,
			vk_beta_y2,
			vk_gamma_x1,
			vk_gamma_x2,
			vk_gamma_y1,
			vk_gamma_y2,
			vk_delta_x1,
			vk_delta_x2,
			vk_delta_y1,
			vk_delta_y2,
			vk_ic_,
		);

		let verified = match Groth16::<ark_bn254::Bn254>::verify(&vk, &pub_inputs, &proof) {
			Ok(v) => v,
			Err(_e) => return Ok((ExitSucceed::Stopped, [].to_vec())),
		};
		#[cfg(feature = "std")]
		let _ = debug::println(format!("Verify result {verified:?}"));

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

#[cfg(feature = "std")]
mod debug {
	use std::{
		io,
		io::{stdout, Write},
	};

	pub fn println(s: String) -> io::Result<()> {
		stdout().write_all((s + "\n").as_bytes())
	}
}
