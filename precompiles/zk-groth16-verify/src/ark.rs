use num_bigint::BigUint;

pub fn ark_bn254_g1(x: &[u8], y: &[u8]) -> Option<ark_bn254::g1::G1Affine> {
	let x_int = BigUint::from_bytes_be(x);
	let y_int = BigUint::from_bytes_be(y);

	let ark_x = ark_ff::BigInt::try_from(x_int).ok()?;
	let ark_y = ark_ff::BigInt::try_from(y_int).ok()?;

	Some(ark_bn254::g1::G1Affine::new(
		ark_bn254::Fq::new(ark_x),
		ark_bn254::Fq::new(ark_y),
	))
}

pub fn ark_bn254_g2(x1: &[u8], x2: &[u8], y1: &[u8], y2: &[u8]) -> Option<ark_bn254::g2::G2Affine> {
	let x1_int = BigUint::from_bytes_be(x1);
	let x2_int = BigUint::from_bytes_be(x2);
	let y1_int = BigUint::from_bytes_be(y1);
	let y2_int = BigUint::from_bytes_be(y2);

	let ark_x1 = ark_ff::BigInt::try_from(x1_int).ok()?;
	let ark_x2 = ark_ff::BigInt::try_from(x2_int).ok()?;
	let ark_y1 = ark_ff::BigInt::try_from(y1_int).ok()?;
	let ark_y2 = ark_ff::BigInt::try_from(y2_int).ok()?;

	Some(ark_bn254::g2::G2Affine::new(
		ark_bn254::Fq2::new(ark_bn254::Fq::new(ark_x1), ark_bn254::Fq::new(ark_x2)),
		ark_bn254::Fq2::new(ark_bn254::Fq::new(ark_y1), ark_bn254::Fq::new(ark_y2)),
	))
}

pub fn ark_bn254_fr(b: &[u8]) -> ark_bn254::Fr {
	ark_bn254::Fr::from(BigUint::from_bytes_be(b))
}
