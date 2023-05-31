use pallet_evm::{ExitSucceed, LinearCostPrecompile, PrecompileFailure};

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;

use crate::precompiles::testp::ark::{ark_bn254_proof, ark_bn254_pub_inputs, ark_bn254_vk};
use ethabi::Token;

pub struct ZKGroth16Verify;

mod ark;

impl LinearCostPrecompile for ZKGroth16Verify {
	const BASE: u64 = 60;
	const WORD: u64 = 12;

	fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
		let types = &[
			ethabi::ParamType::Array(Box::from(ethabi::ParamType::Uint(256))), // input
			ethabi::ParamType::Uint(256),                                      // proof a x
			ethabi::ParamType::Uint(256),                                      // proof a y
			ethabi::ParamType::Uint(256),                                      // proof b x1
			ethabi::ParamType::Uint(256),                                      // proof b x2
			ethabi::ParamType::Uint(256),                                      // proof b y1
			ethabi::ParamType::Uint(256),                                      // proof b y2
			ethabi::ParamType::Uint(256),                                      // proof c x
			ethabi::ParamType::Uint(256),                                      // proof c y
			ethabi::ParamType::Uint(256),                                      // vk alpha x
			ethabi::ParamType::Uint(256),                                      // vk alpha y
			ethabi::ParamType::Uint(256),                                      // vk beta x1
			ethabi::ParamType::Uint(256),                                      // vk beta x2
			ethabi::ParamType::Uint(256),                                      // vk beta y1
			ethabi::ParamType::Uint(256),                                      // vk beta y2
			ethabi::ParamType::Uint(256),                                      // vk gamma x1
			ethabi::ParamType::Uint(256),                                      // vk gamma x2
			ethabi::ParamType::Uint(256),                                      // vk gamma y1
			ethabi::ParamType::Uint(256),                                      // vk gamma y2
			ethabi::ParamType::Uint(256),                                      // vk delta x1
			ethabi::ParamType::Uint(256),                                      // vk delta x2
			ethabi::ParamType::Uint(256),                                      // vk delta y1
			ethabi::ParamType::Uint(256),                                      // vk delta y2
			ethabi::ParamType::Array(Box::from(ethabi::ParamType::FixedArray(
				Box::from(ethabi::ParamType::Uint(256)),
				2,
			))), // vk ic
		];
		let decoded = match ethabi::decode(types, &input[4..]) {
			Ok(v) => v,
			Err(_e) => return Ok((ExitSucceed::Stopped, [].to_vec())),
		};

		#[cfg(feature = "std")]
		let result = types
			.iter()
			.zip(decoded.iter())
			.map(|(ty, to)| format!("{ty} {to}"))
			.collect::<Vec<String>>()
			.join("\n");
		#[cfg(feature = "std")]
		let _ = debug::println(format!("ethabi {:?}", result));

		let pub_inputs = ark_bn254_pub_inputs(
			decoded[0]
				.clone()
				.into_array()
				.unwrap()
				.iter()
				.map(|t| ethabi::encode(&[t.clone()]))
				.collect(),
		);

		let proof = ark_bn254_proof(
			ethabi::encode(&[decoded[1].clone()]).as_slice(),
			ethabi::encode(&[decoded[2].clone()]).as_slice(),
			ethabi::encode(&[decoded[3].clone()]).as_slice(),
			ethabi::encode(&[decoded[4].clone()]).as_slice(),
			ethabi::encode(&[decoded[5].clone()]).as_slice(),
			ethabi::encode(&[decoded[6].clone()]).as_slice(),
			ethabi::encode(&[decoded[7].clone()]).as_slice(),
			ethabi::encode(&[decoded[8].clone()]).as_slice(),
		);

		let mut vk_ic_: Vec<[Vec<u8>; 2]> = Vec::new();

		for point in decoded[23].clone().into_array().unwrap() {
			let array = point.clone().into_fixed_array().unwrap();

			let t1 = array[0].clone();
			let t2 = array[1].clone();

			let x = ethabi::encode(&[t1]);
			let y = ethabi::encode(&[t2]);

			vk_ic_.push([x, y]);
		}

		let vk = ark_bn254_vk(
			ethabi::encode(&[decoded[9].clone()]).as_slice(),
			ethabi::encode(&[decoded[10].clone()]).as_slice(),
			ethabi::encode(&[decoded[11].clone()]).as_slice(),
			ethabi::encode(&[decoded[12].clone()]).as_slice(),
			ethabi::encode(&[decoded[13].clone()]).as_slice(),
			ethabi::encode(&[decoded[14].clone()]).as_slice(),
			ethabi::encode(&[decoded[15].clone()]).as_slice(),
			ethabi::encode(&[decoded[16].clone()]).as_slice(),
			ethabi::encode(&[decoded[17].clone()]).as_slice(),
			ethabi::encode(&[decoded[18].clone()]).as_slice(),
			ethabi::encode(&[decoded[19].clone()]).as_slice(),
			ethabi::encode(&[decoded[20].clone()]).as_slice(),
			ethabi::encode(&[decoded[21].clone()]).as_slice(),
			ethabi::encode(&[decoded[22].clone()]).as_slice(),
			vk_ic_,
		);

		let verified = match Groth16::<ark_bn254::Bn254>::verify(&vk, &pub_inputs, &proof) {
			Ok(t) => t,
			Err(_e) => return Ok((ExitSucceed::Stopped, [].to_vec())),
		};
		#[cfg(feature = "std")]
		let _ = debug::println(format!("Verify result {:?}", verified));

		let encoded = ethabi::encode(&[Token::Bool(verified)]);
		Ok((ExitSucceed::Returned, encoded.to_vec()))
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
