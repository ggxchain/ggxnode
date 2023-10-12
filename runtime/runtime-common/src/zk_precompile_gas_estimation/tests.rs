use super::*;
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
	accounts.insert(
		// H160 address of Alice dev account
		// Derived from SS58 (42 prefix) address
		// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
		// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
		H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
			.expect("internal H160 is valid; qed"),
		fp_evm::GenesisAccount {
			balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
				.expect("internal U256 is valid; qed"),
			code: Default::default(),
			nonce: Default::default(),
			storage: Default::default(),
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
		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("344113499780097036133410144688544669449665038469801025061703634064197059916").unwrap(),
			U256::from_dec_str("8339013355837829034186604529242237434382457535922383671882155606562413431689").unwrap(),
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("6694199005832176626648387473003377778002164049654344388623160500074797775370").unwrap(),
				U256::from_dec_str("6246558478856965861562784864034844546822126660127014237507763330640717221466").unwrap(),
			],
			vec![
				U256::from_dec_str("456859418727494804887897697519797382313397623366843307513728588676215628797").unwrap(),
				U256::from_dec_str("19972929684979882956825623277042855987039264105600158828132263230333203670106").unwrap(),
			],
		];

		let proof_c = vec![
			U256::from_dec_str("8991876521515422964548874095107781976525191569601242691722207742733698821028").unwrap(),
			U256::from_dec_str("17304897036279645989594099459944208131107988079437513579584585106093260802053").unwrap(),
		];

		let vk_alpha = vec![
			U256::from_dec_str("16587578668155053320776501244706909164198053583863284386206654068019392556543").unwrap(),
			U256::from_dec_str("17489859882312386925030082840618136961844042944104008122175974338708995824981").unwrap(),
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("8243058085082563940310351321464397410632520885003827137093771369586433446018").unwrap(),
				U256::from_dec_str("1806964369040712598415691188975966807767394918682209427183228092832571919497").unwrap(),
			],
			vec![
				U256::from_dec_str("5238575049648595236335310308987023404502439315997088611926714272919127054029").unwrap(),
				U256::from_dec_str("10529242360048105445622302340100671863366518517961350431989359465371013288044").unwrap(),
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("422118731661843716781379598081446048950684758761078928178953272944418614265").unwrap(),
				U256::from_dec_str("8071137901873450516865398890359069534977349842235015782367912081100468370342").unwrap(),
			],
			vec![
				U256::from_dec_str("6616051543776580097026224309598118359974125974677691608309070965923230091481").unwrap(),
				U256::from_dec_str("3144026861846828201735234261898216197057552604232403108299101704912138104081").unwrap(),
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("15967728838725285670062386536689775286816477877862913251436519988213464788203").unwrap(),
				U256::from_dec_str("14482652023948193853731701210376245407114581664800773915003137369238699817066").unwrap(),
			],
			vec![
				U256::from_dec_str("20222564526826997580988353930405539667760709452060576182511364617433825375503").unwrap(),
				U256::from_dec_str("1855904408322765948314758770516591135802311025429227172106073057339165107921").unwrap(),
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("2234398803815490360326379119253073021265347911436036657996997085101774002925").unwrap(),
				U256::from_dec_str("10214773739303944856355679056757071889148916152665035343340628435407174724717").unwrap(),
			],
			vec![
				U256::from_dec_str("21500623561133929663133180424780657403302270303706782392356903439186747104812").unwrap(),
				U256::from_dec_str("19328419280811286038333929820672543691407985372391054082759992875889788951230").unwrap(),
			]
		];

		// let valid_input = vec![u64s_to_u256(vec![1250025000])];
		let valid_input = vec![u64s_to_u256(vec![
			14321691860995553260,
			7152862679273281751,
			12752615512303817990,
			1576113262537949146,
		])];

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
				let used_gas = info.used_gas;
				println!("output result {output:?} used gas: {used_gas:?}");
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
