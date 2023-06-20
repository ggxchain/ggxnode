use num_bigint::BigUint;

extern crate alloc;
use alloc::vec::Vec;

pub fn ark_bn254_g1(x: &[u8], y: &[u8]) -> ark_bn254::g1::G1Affine {
	let x_int = BigUint::from_bytes_be(x);
	let y_int = BigUint::from_bytes_be(y);

	let ark_x = ark_ff::BigInt::try_from(x_int).unwrap();
	let ark_y = ark_ff::BigInt::try_from(y_int).unwrap();

	ark_bn254::g1::G1Affine::new(ark_bn254::Fq::new(ark_x), ark_bn254::Fq::new(ark_y))
}

pub fn ark_bn254_g2(x1: &[u8], x2: &[u8], y1: &[u8], y2: &[u8]) -> ark_bn254::g2::G2Affine {
	let x1_int = BigUint::from_bytes_be(x1);
	let x2_int = BigUint::from_bytes_be(x2);
	let y1_int = BigUint::from_bytes_be(y1);
	let y2_int = BigUint::from_bytes_be(y2);

	let ark_x1 = ark_ff::BigInt::try_from(x1_int).unwrap();
	let ark_x2 = ark_ff::BigInt::try_from(x2_int).unwrap();
	let ark_y1 = ark_ff::BigInt::try_from(y1_int).unwrap();
	let ark_y2 = ark_ff::BigInt::try_from(y2_int).unwrap();

	ark_bn254::g2::G2Affine::new(
		ark_bn254::Fq2::new(ark_bn254::Fq::new(ark_x1), ark_bn254::Fq::new(ark_x2)),
		ark_bn254::Fq2::new(ark_bn254::Fq::new(ark_y1), ark_bn254::Fq::new(ark_y2)),
	)
}

#[allow(clippy::too_many_arguments)]
pub fn ark_bn254_proof(
	a_x: &[u8],
	a_y: &[u8],
	b_x1: &[u8],
	b_x2: &[u8],
	b_y1: &[u8],
	b_y2: &[u8],
	c_x: &[u8],
	c_y: &[u8],
) -> ark_groth16::Proof<ark_bn254::Bn254> {
	let a = ark_bn254_g1(a_x, a_y);
	let c = ark_bn254_g1(c_x, c_y);

	let b = ark_bn254_g2(b_x1, b_x2, b_y1, b_y2);

	ark_groth16::Proof { a, b, c }
}

#[allow(clippy::too_many_arguments)]
pub fn ark_bn254_vk(
	alpha_x: &[u8],
	alpha_y: &[u8],
	beta_x1: &[u8],
	beta_x2: &[u8],
	beta_y1: &[u8],
	beta_y2: &[u8],
	gamma_x1: &[u8],
	gamma_x2: &[u8],
	gamma_y1: &[u8],
	gamma_y2: &[u8],
	delta_x1: &[u8],
	delta_x2: &[u8],
	delta_y1: &[u8],
	delta_y2: &[u8],
	ic: Vec<[Vec<u8>; 2]>,
) -> ark_groth16::VerifyingKey<ark_bn254::Bn254> {
	let alpha_g1 = ark_bn254_g1(alpha_x, alpha_y);

	let beta_g2 = ark_bn254_g2(beta_x1, beta_x2, beta_y1, beta_y2);
	let gamma_g2 = ark_bn254_g2(gamma_x1, gamma_x2, gamma_y1, gamma_y2);
	let delta_g2 = ark_bn254_g2(delta_x1, delta_x2, delta_y1, delta_y2);

	let gamma_abc_g1 = ic.iter().map(|p| ark_bn254_g1(&p[0], &p[1])).collect();

	ark_groth16::VerifyingKey {
		alpha_g1,
		beta_g2,
		gamma_g2,
		delta_g2,
		gamma_abc_g1,
	}
}

pub fn ark_bn254_fr(b: &[u8]) -> ark_bn254::Fr {
	ark_bn254::Fr::from(BigUint::from_bytes_be(b))
}
