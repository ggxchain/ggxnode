use crate::zk_precompile_gas_estimation::mock::*;
use fp_evm::GenesisAccount;
use frame_support::{
	assert_ok,
	traits::{GenesisBuild, LockIdentifier, LockableCurrency, WithdrawReasons},
};
use pallet_evm::GenesisConfig;
use pallet_evm_precompile_zk_groth16_verify::Action;
use precompile_utils::{testing::*, EvmDataWriter};
use sp_core::{H160, U256};
use std::{collections::BTreeMap, str::FromStr};

fn precompiles() -> MockPrecompileSet<Test> {
	MockPrecompiles::get()
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let mut accounts = BTreeMap::new();
	accounts.insert(
		H160::from_str("1000000000000000000000000000000000000001").unwrap(),
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::from(1000000),
			storage: Default::default(),
			code: vec![
				0x00, // STOP
			],
		},
	);
	accounts.insert(
		H160::from_str("1000000000000000000000000000000000000002").unwrap(),
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::from(1000000),
			storage: Default::default(),
			code: vec![
				0xff, // INVALID
			],
		},
	);
	accounts.insert(
		H160::default(), // root
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::max_value(),
			storage: Default::default(),
			code: vec![],
		},
	);

	pallet_balances::GenesisConfig::<Test> {
		// Create the block author account with some balance.
		balances: vec![(
			H160::from_str("0x1234500000000000000000000000000000000000").unwrap(),
			12345,
		)],
	}
	.assimilate_storage(&mut t)
	.expect("Pallet balances storage can be assimilated");
	GenesisBuild::<Test>::assimilate_storage(&GenesisConfig { accounts }, &mut t).unwrap();
	t.into()
}

// #[test]
// fn zk_groth16_verify_work() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let (
// 			proof_a,
// 			proof_b,
// 			proof_c,
// 			vk_alpha,
// 			vk_beta,
// 			vk_gamma,
// 			vk_delta,
// 			vk_ic,
// 			valid_input,
// 			_,
// 		) = generate_test_case_data().unwrap();

// 		precompiles()
// 			.prepare_test(
// 				TestAccount::Alice,
// 				PRECOMPILE_ADDRESS,
// 				EvmDataWriter::new_with_selector(Action::Verify)
// 					.write(proof_a)
// 					.write(proof_b)
// 					.write(proof_c)
// 					.write(vk_alpha)
// 					.write(vk_beta)
// 					.write(vk_gamma)
// 					.write(vk_delta)
// 					.write(vk_ic)
// 					.write(valid_input)
// 					.build(),
// 			)
// 			.expect_no_logs()
// 			.execute_returns(EvmDataWriter::new().write(true).build());
// 	})
// }

// #[test]
// fn zk_groth16_verify_work_with_invalid_proof() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let (
// 			proof_a,
// 			proof_b,
// 			proof_c,
// 			vk_alpha,
// 			vk_beta,
// 			vk_gamma,
// 			vk_delta,
// 			vk_ic,
// 			_,
// 			invalid_input,
// 		) = generate_test_case_data().unwrap();

// 		precompiles()
// 			.prepare_test(
// 				TestAccount::Alice,
// 				PRECOMPILE_ADDRESS,
// 				EvmDataWriter::new_with_selector(Action::Verify)
// 					.write(proof_a)
// 					.write(proof_b)
// 					.write(proof_c)
// 					.write(vk_alpha)
// 					.write(vk_beta)
// 					.write(vk_gamma)
// 					.write(vk_delta)
// 					.write(vk_ic)
// 					.write(invalid_input)
// 					.build(),
// 			)
// 			.expect_no_logs()
// 			.execute_returns(EvmDataWriter::new().write(false).build());
// 	})
// }

// fn generate_test_case_data() -> Result<
// 	(
// 		(U256, U256),
// 		((U256, U256), (U256, U256)),
// 		(U256, U256),
// 		(U256, U256),
// 		((U256, U256), (U256, U256)),
// 		((U256, U256), (U256, U256)),
// 		((U256, U256), (U256, U256)),
// 		Vec<(U256, U256)>,
// 		Vec<U256>,
// 		Vec<U256>,
// 	),
// 	Box<dyn std::error::Error>,
// > {
// 	let proof_a = decode_g1_point(
// 		"13202079600221154376862161785979680082984660469505374274880948735521253479994",
// 		"19032139815435908179959144311759562497239236177745989139113028703727512477837",
// 	)?;
// 	let proof_b = decode_g2_point(
// 		"9517359327043802798811688827065407805934924568686293993682568334305900037151",
// 		"13975418982386111217378923290980800393212535787789845393400867460398182717751",
// 		"11101434469251848949317000686121782094334155840067455941163819739470030872205",
// 		"3351121397470969456277617123820147601817413346203636355523709813813837616699",
// 	)?;
// 	let proof_c = decode_g1_point(
// 		"21771166379144524714497801611702430117390298454683954881352912868492853507834",
// 		"5971832614272362565584439633663845994795381011258125087840397908182066694531",
// 	)?;

// 	let vk_alpha = decode_g1_point(
// 		"7318409901911144874440195167086183143676595981815053389579728623121590098440",
// 		"18845965879715444612950452554360629789407129470518446134938217746489723713219",
// 	)?;
// 	let vk_beta = decode_g2_point(
// 		"4640649673239597789758809808535118135578677216672702870175791505196312738305",
// 		"13141288066376351908866878766575256664575916601245245304316354941350328880142",
// 		"11492338667195076401975872253943030431149343004937779351839311477974294172860",
// 		"17604387530215185597479117283681563543587978658046512716245947218617178983155",
// 	)?;
// 	let vk_gamma = decode_g2_point(
// 		"10857046999023057135944570762232829481370756359578518086990519993285655852781",
// 		"11559732032986387107991004021392285783925812861821192530917403151452391805634",
// 		"8495653923123431417604973247489272438418190587263600148770280649306958101930",
// 		"4082367875863433681332203403145435568316851327593401208105741076214120093531",
// 	)?;
// 	let vk_delta = decode_g2_point(
// 		"5882870888685857628232224840789532289346124290586616915986585508513239272539",
// 		"8206718104089392401855946495573733123991363841198873660903571227166120193870",
// 		"14275677868038957349366208693756706908778821863795564855498136614399516409168",
// 		"20950579407520036072561845357324335488555384097745021047033651867265123837403",
// 	)?;
// 	let vk_ic = decode_ic(vec![
// 		[
// 			"15329034480187562940265095627808115353397553736992059710948268284574612609224"
// 				.to_string(),
// 			"13272704791638435782238987852007128987814629753205340563304933194747762248428"
// 				.to_string(),
// 		],
// 		[
// 			"17269839325091679315052274785558946544729609490743199699197195008879157661695"
// 				.to_string(),
// 			"4142750859697696641705372803120309740931359230261851701215055719438325633654"
// 				.to_string(),
// 		],
// 	])?;

// 	let valid_input: Vec<U256> = vec![U256::from(66)];
// 	let invalid_input: Vec<U256> = vec![U256::from(65)];

// 	Ok((
// 		proof_a,
// 		proof_b,
// 		proof_c,
// 		vk_alpha,
// 		vk_beta,
// 		vk_gamma,
// 		vk_delta,
// 		vk_ic,
// 		valid_input,
// 		invalid_input,
// 	))
// }

// fn decode_g1_point(x: &str, y: &str) -> Result<(U256, U256), Box<dyn std::error::Error>> {
// 	Ok((U256::from_dec_str(x)?, U256::from_dec_str(y)?))
// }

// fn decode_g2_point(
// 	x1: &str,
// 	x2: &str,
// 	y1: &str,
// 	y2: &str,
// ) -> Result<((U256, U256), (U256, U256)), Box<dyn std::error::Error>> {
// 	Ok((
// 		(U256::from_dec_str(x1)?, U256::from_dec_str(x2)?),
// 		(U256::from_dec_str(y1)?, U256::from_dec_str(y2)?),
// 	))
// }

// fn decode_ic(points: Vec<[String; 2]>) -> Result<Vec<(U256, U256)>, Box<dyn std::error::Error>> {
// 	let mut result = Vec::new();
// 	for point in points {
// 		result.push((
// 			U256::from_dec_str(&point[0])?,
// 			U256::from_dec_str(&point[1])?,
// 		));
// 	}
// 	Ok(result)
// }
