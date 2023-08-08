use crate::zk_precompile_gas_estimation::mock::*;
use ethabi::Token;
use fp_evm::GenesisAccount;
use frame_support::traits::GenesisBuild;
use pallet_evm::{GenesisConfig, Runner};
use sp_core::{H160, U256};
use std::{collections::BTreeMap, str::FromStr};
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let mut accounts = BTreeMap::new();
	accounts.insert(
		H160::from_str("1000000000000000000000000000000000000001").unwrap(),
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::max_value(),
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
			balance: U256::max_value(),
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
	accounts.insert(
		H160::from_str("1000000000000000000000000000000000000666").unwrap(),
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::max_value(),
			storage: Default::default(),
			code: vec![], // No code, this is an EOA
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

#[test]
fn test_zk_precompile_gas_estimation() {
	new_test_ext().execute_with(|| {
		let caller = "1000000000000000000000000000000000000666".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("17526830733396711760614584592227592290041884054274692385786120317424586951928").unwrap(),
			U256::from_dec_str("14572975009604791670190722144346966861775781663452799801994174625223530481293").unwrap(),
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("601091821423560000095338178586024773386806597229060347992230108339560337314").unwrap(),
				U256::from_dec_str("20742329792185135417418009297070484022157549629159037177761763368870046435272").unwrap(),
			],
			vec![
				U256::from_dec_str("16755958476414893111742804329540858423617749911023931584301476920013641667491").unwrap(),
				U256::from_dec_str("17783640965723747611121456292908504762681694379312402130038762071821040335901").unwrap(),
			],
		];

		let proof_c = vec![
			U256::from_dec_str("16585885402862686337557433720680035430031425737321159047915573958245961596065").unwrap(),
			U256::from_dec_str("21138178351445807771687394250843863988645614435175735346332333179172518378873").unwrap(),
		];

		let vk_alpha = vec![
			U256::from_dec_str("11484345157460561328461496884344190739368158625792198603002887108334722367669").unwrap(),
			U256::from_dec_str("7086541917963105127497541193167841067353924271429224233965614329678507699488").unwrap(),
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("11233128783379821052483619196929223312692971108992242120588689458439987752067").unwrap(),
				U256::from_dec_str("18640573156008262049228946668908337221222445157240294147034515643976275941832").unwrap(),
			],
			vec![
				U256::from_dec_str("8036983389612370781023375187819482868320454529134623464035669006179478819783").unwrap(),
				U256::from_dec_str("16424116254718275500394502257751813449316936907205655002329084814392003619802").unwrap(),
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("11228532404788151396730538740677955321165465678604965636022681521723486460585").unwrap(),
				U256::from_dec_str("9126453034411305688949328104662278794713000842053283268128161824572984268064").unwrap(),
			],
			vec![
				U256::from_dec_str("6386111381608251566396732981213051438889552027469318069778613233664780673905").unwrap(),
				U256::from_dec_str("20548857422827059363546153463236239952243300189763734417419279501301392340910").unwrap(),
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("16318831326794673155348007016625372642245174324642568461921219640531003953421").unwrap(),
				U256::from_dec_str("10357000493091964334208434777562330142557973636508422501741235212619215377564").unwrap(),
			],
			vec![
				U256::from_dec_str("2549252377666082313659764548733755762481620259406193451284130401099909523405").unwrap(),
				U256::from_dec_str("17286438687897881040082074476507835506847599850151991459812945029157729581277").unwrap(),
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("19809018900500753691845636946336582450384254173888544359470795703770668108982").unwrap(),
				U256::from_dec_str("18905865743072988362439446807054892183348768606005293094676173646899256980681").unwrap(),
			],
			vec![
				U256::from_dec_str("13731638277857114344416462118077651769684398274209992879843147837262038701628").unwrap(),
				U256::from_dec_str("19144851605913607756613362053945586479161724714752749470233682046477131900096").unwrap(),
			]
		];

		let valid_input = vec![
			U256::from_dec_str("1250025000").unwrap()
		];
		// 		let valid_input = vec![
		// 	U256::from_dec_str("14965631224775206224").unwrap(),
		// 	U256::from_dec_str("3021577815302938909").unwrap(),
		// 	U256::from_dec_str("14359293880404272991").unwrap(),
		// 	U256::from_dec_str("1555005537055779113").unwrap(),
		// ];
		let mut encoded_call = vec![0u8; 4];
		encoded_call[0..4].copy_from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..4]);
		println!("encoded_call: {:?}", encoded_call);
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
		println!("parameters: {:?}", parameters);
		encoded_call.extend(parameters);
		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
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
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				println!("output result {output:?}");
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					println!("Verification result {result:?}");
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				println!("Error: {:?}", e);
				assert!(false);
			}
		}
	});
}
