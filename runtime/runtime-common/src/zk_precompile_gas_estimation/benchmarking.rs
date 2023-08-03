// #![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
// frontier
use pallet_evm::Runner;
use sp_std::prelude::*;
benchmarks! {
	demo_runner {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "1000000000000000000000000000000000000666".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("13202079600221154376862161785979680082984660469505374274880948735521253479994").unwrap(),
			U256::from_dec_str("19032139815435908179959144311759562497239236177745989139113028703727512477837").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("9517359327043802798811688827065407805934924568686293993682568334305900037151").unwrap(),
				U256::from_dec_str("13975418982386111217378923290980800393212535787789845393400867460398182717751").unwrap()
			],
			vec![
				U256::from_dec_str("11101434469251848949317000686121782094334155840067455941163819739470030872205").unwrap(),
				U256::from_dec_str("3351121397470969456277617123820147601817413346203636355523709813813837616699").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("21771166379144524714497801611702430117390298454683954881352912868492853507834").unwrap(),
			U256::from_dec_str("5971832614272362565584439633663845994795381011258125087840397908182066694531").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("7318409901911144874440195167086183143676595981815053389579728623121590098440").unwrap(),
			U256::from_dec_str("18845965879715444612950452554360629789407129470518446134938217746489723713219").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("4640649673239597789758809808535118135578677216672702870175791505196312738305").unwrap(),
				U256::from_dec_str("13141288066376351908866878766575256664575916601245245304316354941350328880142").unwrap()
			],
			vec![
				U256::from_dec_str("11492338667195076401975872253943030431149343004937779351839311477974294172860").unwrap(),
				U256::from_dec_str("17604387530215185597479117283681563543587978658046512716245947218617178983155").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("10857046999023057135944570762232829481370756359578518086990519993285655852781").unwrap(),
				U256::from_dec_str("11559732032986387107991004021392285783925812861821192530917403151452391805634").unwrap()
			],
			vec![
				U256::from_dec_str("8495653923123431417604973247489272438418190587263600148770280649306958101930").unwrap(),
				U256::from_dec_str("4082367875863433681332203403145435568316851327593401208105741076214120093531").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("5882870888685857628232224840789532289346124290586616915986585508513239272539").unwrap(),
				U256::from_dec_str("8206718104089392401855946495573733123991363841198873660903571227166120193870").unwrap()
			],
			vec![
				U256::from_dec_str("14275677868038957349366208693756706908778821863795564855498136614399516409168").unwrap(),
				U256::from_dec_str("20950579407520036072561845357324335488555384097745021047033651867265123837403").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("15329034480187562940265095627808115353397553736992059710948268284574612609224").unwrap(),
				U256::from_dec_str("13272704791638435782238987852007128987814629753205340563304933194747762248428").unwrap()
			],
			vec![
				U256::from_dec_str("17269839325091679315052274785558946544729609490743199699197195008879157661695").unwrap(),
				U256::from_dec_str("4142750859697696641705372803120309740931359230261851701215055719438325633654").unwrap()
			]
		];

		let valid_input = vec![U256::from(66)];
		let mut encoded_call = vec![0u8; 4];
		encoded_call[0..4].copy_from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..4]);
		let parameters = ethabi::encode(&[
			Token::FixedArray(proof_a.into_iter().map(Token::Uint).collect()),
			Token::FixedArray(proof_b.into_iter().map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect())).collect()),
			Token::FixedArray(proof_c.into_iter().map(Token::Uint).collect()),
			Token::FixedArray(vk_alpha.into_iter().map(Token::Uint).collect()),
			Token::FixedArray(vk_beta.into_iter().map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect())).collect()),
			Token::FixedArray(vk_gamma.into_iter().map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect())).collect()),
			Token::FixedArray(vk_delta.into_iter().map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect())).collect()),
			Token::Array(vk_ic.into_iter().map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect())).collect()),
			Token::Array(valid_input.into_iter().map(Token::Uint).collect()),
		]);
		encoded_call.extend(parameters);
		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <Test as pallet_evm::Config>::Runner::call(
			caller,
			contract_address,
			encoded_call,
			value,
			gas_limit_call,
			Some(U256::from(1_000_000_000)),
			Some(U256::from(1_000_000_000)),
			None,
			Vec::new(),
			is_transactional,
			validate,
			<Test as pallet_evm::Config>::config(),
		);
		log::info!(
			target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
			"Benchmarking call end.",
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"output result {:?}",
					output
				);
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					log::info!(
						target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
						"Verification result {result:?}",
					);
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"Benchmarking failed",
				);
			}
		}
	}
}
impl_benchmark_test_suite!(
	Pallet,
	crate::zk_precompile_gas_estimation::tests::new_test_ext(),
	crate::zk_precompile_gas_estimation::mock::Test
);
