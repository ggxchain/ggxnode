use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
// frontier
use ethabi::Token;
use frame_benchmarking::vec;
use pallet_evm::Runner;
use sp_core::{H256, U256};
use sp_std::prelude::*;

struct ZKPrecompileVerifier {
	selector: H256,
	verifying_key: VerifyingKey,
}

struct VerifyingKey {
	proof: Proof,
	vk: VerifyingKeyComponents,
	public_input: Vec<U256>,
}

struct Proof {
	a: Vec<U256>,
	b: Vec<Vec<U256>>,
	c: Vec<U256>,
}

struct VerifyingKeyComponents {
	alpha: Vec<U256>,
	beta: Vec<Vec<U256>>,
	gamma: Vec<Vec<U256>>,
	delta: Vec<Vec<U256>>,
	ic: Vec<Vec<U256>>,
}

impl ZKPrecompileVerifier {
	pub fn new(selector: H256, verifying_key: VerifyingKey) -> Self {
		Self {
			selector,
			verifying_key,
		}
	}
}

impl VerifyingKey {
	pub fn new(proof: Proof, vk: VerifyingKeyComponents, public_input: Vec<U256>) -> Self {
		Self {
			proof,
			vk,
			public_input,
		}
	}
}

impl Proof {
	pub fn new(a: Vec<U256>, b: Vec<Vec<U256>>, c: Vec<U256>) -> Self {
		Self { a, b, c }
	}
}

impl VerifyingKeyComponents {
	pub fn new(
		alpha: Vec<U256>,
		beta: Vec<Vec<U256>>,
		gamma: Vec<Vec<U256>>,
		delta: Vec<Vec<U256>>,
		ic: Vec<Vec<U256>>,
	) -> Self {
		Self {
			alpha,
			beta,
			gamma,
			delta,
			ic,
		}
	}
}

impl ZKPrecompileVerifier {
	pub fn generate_benchmarking_parameters(&self) -> Vec<u8> {
		let mut encoded_call = vec![0u8; 4];
		encoded_call[0..4].copy_from_slice(&self.selector.as_bytes()[0..4]);

		let parameters = ethabi::encode(&[
			Token::FixedArray(
				self.verifying_key
					.proof
					.a
					.clone()
					.into_iter()
					.map(Token::Uint)
					.collect(),
			),
			Token::FixedArray(
				self.verifying_key
					.proof
					.b
					.clone()
					.into_iter()
					.map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect()))
					.collect(),
			),
			Token::FixedArray(
				self.verifying_key
					.proof
					.c
					.clone()
					.into_iter()
					.map(Token::Uint)
					.collect(),
			),
			Token::FixedArray(
				self.verifying_key
					.vk
					.alpha
					.clone()
					.into_iter()
					.map(Token::Uint)
					.collect(),
			),
			Token::FixedArray(
				self.verifying_key
					.vk
					.beta
					.clone()
					.into_iter()
					.map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect()))
					.collect(),
			),
			Token::FixedArray(
				self.verifying_key
					.vk
					.gamma
					.clone()
					.into_iter()
					.map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect()))
					.collect(),
			),
			Token::FixedArray(
				self.verifying_key
					.vk
					.delta
					.clone()
					.into_iter()
					.map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect()))
					.collect(),
			),
			Token::Array(
				self.verifying_key
					.vk
					.ic
					.clone()
					.into_iter()
					.map(|inner| Token::FixedArray(inner.into_iter().map(Token::Uint).collect()))
					.collect(),
			),
			Token::Array(
				self.verifying_key
					.public_input
					.clone()
					.into_iter()
					.map(Token::Uint)
					.collect(),
			),
		]);
		encoded_call.extend(parameters);
		encoded_call
	}
}

benchmarks! {
	nconstraints_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("323654832133639084097738656057999026789295280666886852594984576709163539526").unwrap(),
				U256::from_dec_str("6142957218280645039826855630872622613337777761157366378615236951849913775542").unwrap()
			],
			vec![
				vec![
					U256::from_dec_str("3781402603457401622553078625624300084337446058890012151530042647448923283156").unwrap(),
					U256::from_dec_str("3517436796495769146106898128956048233729888622957356948216773257280783319311").unwrap()
				],
				vec![
					U256::from_dec_str("18697717875318192535466611481750571094853863458428844875498420533596881568336").unwrap(),
					U256::from_dec_str("21426423909981695530725752097567643188095093768132522705564475372400085021436").unwrap()
				],
			],
			vec![
				U256::from_dec_str("21613404883582191576526635676329374200679201344239432919502376530553941450391").unwrap(),
				U256::from_dec_str("15157755445966041962464884593780030869511182260822371637833098723763718514455").unwrap()
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("11521346682463921906610271064518766485088278761420928386309550940286650671457").unwrap(),
				U256::from_dec_str("14352830518175561380348856554041231805603495989169432000725584250110291361678").unwrap()
			],
			vec![
				vec![
					U256::from_dec_str("15782596638536803844844408481637582589736413871396993858917786753770876529739").unwrap(),
					U256::from_dec_str("14911823231521430202833448929652297782496660742577343858219844871404504144036").unwrap()
				],
				vec![
					U256::from_dec_str("2082222914940990020573243578958386229340150899029559516350605378857473031525").unwrap(),
					U256::from_dec_str("13498996195230132373906915048494917069165130851394909752977335076805840989307").unwrap()
				],
			],
			vec![
				vec![
					U256::from_dec_str("3598021131780341035246011324427156192540414244243113294885735792953741606880").unwrap(),
					U256::from_dec_str("2036084172531515709709771052774738245922046202318890723044728246225539963428").unwrap()
				],
				vec![
					U256::from_dec_str("5925457711548235099025126547368719549508643015841982506039250719340224619945").unwrap(),
					U256::from_dec_str("6297446610567432982066083461547400065680646594595441716191878482607632761242").unwrap()
				],
			],
			vec![
				vec![
					U256::from_dec_str("8540596677770285018018336727251534074846909511956041125063939097422779191925").unwrap(),
					U256::from_dec_str("4004145476643750015352094786522189637304633549022019774555222068038038169997").unwrap()
				],
				vec![
					U256::from_dec_str("14044576757821380530542904664046256148829410228513509672845139471539338485558").unwrap(),
					U256::from_dec_str("21655834928283484760531472827487408029775629615266614212974138202212982461041").unwrap()
				]
			],
			vec![
				vec![
					U256::from_dec_str("18160015955331442477546361309657533914485780296451278252095683998154112390415").unwrap(),
					U256::from_dec_str("2421052745427024367257863480713357899822722660532800486395996070904226714086").unwrap()
				],
				vec![
					U256::from_dec_str("13418845437340247256566377432948869602549774753220761674399899795516131627829").unwrap(),
					U256::from_dec_str("19073971532401628571532840417056837528958340897410372352356543204393028153748").unwrap()
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			14965631224775206224,
			3021577815302938909,
			14359293880404272991,
			1555005537055779113
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_10 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_10 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_10 Benchmarking failed",
				);
				panic!("nconstraints_10 Benchmarking failed");
			}
		}
	}

	nconstraints_50 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("2903574088029507435491062627919627539459038881203099780902103872696986272650").unwrap(),
				U256::from_dec_str("3875062839032259754068923295563862040166715100637843024855021734684308685930").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5467576156112132028552287163049386912078800648727055176723583005212825192497").unwrap(),
					U256::from_dec_str("14958898784032520052694892656913773236193256742890765604760593633503039104983").unwrap(),
				],
				vec![
					U256::from_dec_str("5396143311660903133040196691443703988892234257628755380984718394981374344158").unwrap(),
					U256::from_dec_str("8621644456104449033213297238562527153800159584768570376273022394943050969320").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("11879092906448699692394725971977132036554363118574468164770659810752018463948").unwrap(),
				U256::from_dec_str("6829624468475499149259780051037762599455296300930683215687755977165243124154").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("12520775544030530008009522467466185193766786263812261579427634884004150619566").unwrap(),
				U256::from_dec_str("5032545863820965741764073435262250124257467813113463153084126384917060899315").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("10017955328073015145877787608777450033090741247853140204265773706371885379542").unwrap(),
					U256::from_dec_str("9658203662920649739629860626443008098625554125855709045942833415049413198251").unwrap(),
				],
				vec![
					U256::from_dec_str("7602415148393817869205609282139785691690714005596249546125364310759985118071").unwrap(),
					U256::from_dec_str("6621887421318408197643297351639235440436194355283660692959196816576578875866").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("5311307398908794283615493653665859916837738758333406097237626345248614605789").unwrap(),
					U256::from_dec_str("17876886843886949913975349493510247353159584223381019718903135647608874168034").unwrap(),
				],
				vec![
					U256::from_dec_str("16871344858870476549165036353120397473700844904231047018564735968613233188965").unwrap(),
					U256::from_dec_str("3780010632071163899826543545524090280002435997376718121100375224910528030644").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("5274939476765287332787933848172441391963317633567336761478263433231602711901").unwrap(),
					U256::from_dec_str("20318717968421229127062334564952813061229602553850186042308931686220563993985").unwrap(),
				],
				vec![
					U256::from_dec_str("18867986549807700609691354300906273266753459571664361845748650324369589087167").unwrap(),
					U256::from_dec_str("3462802759155100024118895949055238081126591020075851877084161838812514424801").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("2073721309366561324099267234284336470258146869696430225377434175843251484752").unwrap(),
					U256::from_dec_str("5562089018027206512286783868306950659834000523781522158661084193254167305831").unwrap(),
				],
				vec![
					U256::from_dec_str("7633925392184781104787537197952383405066326059553608577981951477761067715424").unwrap(),
					U256::from_dec_str("19510227686274170330355317434483857344641524658406254379151352263942714837771").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			14754876691570659999,
			1449725765053302887,
			10573592104590215630,
			2031301759166401468
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_50 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_50 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_50 Benchmarking failed",
				);
				panic!("nconstraints_50 Benchmarking failed");
			}
		}
	}

	nconstraints_1000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("12451505848309631687252015279596976128225375079184616465869275088833975377438").unwrap(),
				U256::from_dec_str("18028472547304696309803026581736164843724278168110632635282700219721294267849").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("724941625340075448308940885419778921267046089963198424690563710257350467820").unwrap(),
					U256::from_dec_str("19418815036545241911278056687101064762643664412732993043969007894248087117385").unwrap(),
				],
				vec![
					U256::from_dec_str("13249414494230793056410904135885722890271124096600389432878053498484425858770").unwrap(),
					U256::from_dec_str("21762806144353143011028620465453743506764842517668851190776375592288870325215").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("9206867270262791130308246301015463866896172966002943363908694359851632501217").unwrap(),
				U256::from_dec_str("13679683495177263967294831832954860495137891909047512983345948806664713075553").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("13896934827344718021853288836918374118104295193491067840322643424358899121542").unwrap(),
				U256::from_dec_str("8854052944916041632001392967979629780372345382861670282089777314383817716115").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("3601177773308419037453361001975316835601350296485805323140167568598797930642").unwrap(),
					U256::from_dec_str("48851914275316782871445347689531673984580545598074561513165366823294923594").unwrap(),
				],
				vec![
					U256::from_dec_str("4538012201084519544875007504928553564968897773565284770715803834936680149613").unwrap(),
					U256::from_dec_str("3155968616270588886660033728776257587454508333510911411658919790925259116614").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("6521538659624675039700866061977650833785775087701556120926232218187885377774").unwrap(),
					U256::from_dec_str("18707623749213430453109215145186606994638201653518117177006249287165414157706").unwrap(),
				],
				vec![
					U256::from_dec_str("11743109857123024729794370527373948778820323338754388805090881243904381703330").unwrap(),
					U256::from_dec_str("3577725017182890090279110701721773090147301853148322630192233820247926772155").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("10697077492206594361644026971284249101590705030465491131467731100150008255694").unwrap(),
					U256::from_dec_str("1114380532795466865933623446272212800793315337032855536172932313989250445726").unwrap(),
				],
				vec![
					U256::from_dec_str("11747915450686416011932172170367549011245903715607283230565719536126230133873").unwrap(),
					U256::from_dec_str("1877153534214274703747923906347594924169181268669702005894131954432202950068").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("6294921440683861296812943370338888809368101163511446375226378176013676840633").unwrap(),
					U256::from_dec_str("8766686502883018606879699065134987161183815270662618961653335871316040755302").unwrap(),
				],
				vec![
					U256::from_dec_str("15422085976834841481597157129530660038025986735930069198619117556493669017719").unwrap(),
					U256::from_dec_str("6824904336141021449652133306590775160232847673735850151493603664420102965477").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			5637188163546619750,
			4876642863872575056,
			11311886144936918135,
			2620236256196171805
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_1000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_1000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_1000 Benchmarking failed",
				);
				panic!("nconstraints_1000 Benchmarking failed");
			}
		}
	}
	nconstraints_900000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("19469224081137364911375441935428613758543844094780123177610680784796374565509").unwrap(),
				U256::from_dec_str("2964109100176825277964902990593130655404587548088911907169158668561756482373").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("1804788596044451078391485097362341934563024367189115611320140606169017734442").unwrap(),
					U256::from_dec_str("13075356869764013404490647878715830765050556805353047586243641659166972614506").unwrap(),
				],
				vec![
					U256::from_dec_str("11510752280195741101381678478126680255679095252133774347633523263393718090827").unwrap(),
					U256::from_dec_str("19917734624109913540474402939707087760514256808495299709893660593520804971329").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("17650234014776935476405426867265189573543251264844619398004397025922351000524").unwrap(),
				U256::from_dec_str("13980409623437481166079302794215538337078177634480935418093274323345259996058").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("7204013913559348808806179272679133668079798990573619252699122941477292720367").unwrap(),
				U256::from_dec_str("14448814309541293146746525253301533902449817953703436076734477345366335651547").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("12755354931522455226769415963087662012592248962914015697167303994293035888664").unwrap(),
					U256::from_dec_str("19979994508677160041392994419804596139216500302977150541381981675855322022114").unwrap(),
				],
				vec![
					U256::from_dec_str("3518578052095992035034011768990132253597152650327354182069223777691356587613").unwrap(),
					U256::from_dec_str("1512363495951297845516866605291301480678745477544041997800458458709067854003").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("21483901865224554977407186643009006049964266844355020377697169106001216486465").unwrap(),
					U256::from_dec_str("18122216236568402305988297032517775914181563813086813006164216689752246508886").unwrap(),
				],
				vec![
					U256::from_dec_str("7100124544075226034251596932805277307734457677256004047412258693103409543678").unwrap(),
					U256::from_dec_str("9484826513489508497326605298995346628011303684771342589708630141744196880174").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("4294809674390523555755416088340227038268056188806697061914357207030835445784").unwrap(),
					U256::from_dec_str("12772882987652274474928251216882135157601158713560023135442912416889883375009").unwrap(),
				],
				vec![
					U256::from_dec_str("5017814360156811257431228283078380210712481496173867244303167538680301406355").unwrap(),
					U256::from_dec_str("5483986251263351055676143301589669624262947345327185314870128974364440001262").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("10485365056956235833573091505977794158366069646492853613385564319176805042067").unwrap(),
					U256::from_dec_str("21560988819791135038184934219494970236802908435926324068683299817562907341696").unwrap(),
				],
				vec![
					U256::from_dec_str("9274251274730510866039258353232655161206943117093211738883330925803202790098").unwrap(),
					U256::from_dec_str("19994917632455570632082635510110118222993755636207706143729050502915579896735").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			10518156496175725870,
			8513656267880232886,
			1469451646407182009,
			3425508166019525724
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_900000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_900000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_900000 Benchmarking failed",
				);
				panic!("nconstraints_900000 Benchmarking failed");
			}
		}
	}

	nconstraints_30_input_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("344113499780097036133410144688544669449665038469801025061703634064197059916").unwrap(),
				U256::from_dec_str("8339013355837829034186604529242237434382457535922383671882155606562413431689").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("6694199005832176626648387473003377778002164049654344388623160500074797775370").unwrap(),
					U256::from_dec_str("6246558478856965861562784864034844546822126660127014237507763330640717221466").unwrap(),
				],
				vec![
					U256::from_dec_str("456859418727494804887897697519797382313397623366843307513728588676215628797").unwrap(),
					U256::from_dec_str("19972929684979882956825623277042855987039264105600158828132263230333203670106").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("8991876521515422964548874095107781976525191569601242691722207742733698821028").unwrap(),
				U256::from_dec_str("17304897036279645989594099459944208131107988079437513579584585106093260802053").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("16587578668155053320776501244706909164198053583863284386206654068019392556543").unwrap(),
				U256::from_dec_str("17489859882312386925030082840618136961844042944104008122175974338708995824981").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("8243058085082563940310351321464397410632520885003827137093771369586433446018").unwrap(),
					U256::from_dec_str("1806964369040712598415691188975966807767394918682209427183228092832571919497").unwrap(),
				],
				vec![
					U256::from_dec_str("5238575049648595236335310308987023404502439315997088611926714272919127054029").unwrap(),
					U256::from_dec_str("10529242360048105445622302340100671863366518517961350431989359465371013288044").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("422118731661843716781379598081446048950684758761078928178953272944418614265").unwrap(),
					U256::from_dec_str("8071137901873450516865398890359069534977349842235015782367912081100468370342").unwrap(),
				],
				vec![
					U256::from_dec_str("6616051543776580097026224309598118359974125974677691608309070965923230091481").unwrap(),
					U256::from_dec_str("3144026861846828201735234261898216197057552604232403108299101704912138104081").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("15967728838725285670062386536689775286816477877862913251436519988213464788203").unwrap(),
					U256::from_dec_str("14482652023948193853731701210376245407114581664800773915003137369238699817066").unwrap(),
				],
				vec![
					U256::from_dec_str("20222564526826997580988353930405539667760709452060576182511364617433825375503").unwrap(),
					U256::from_dec_str("1855904408322765948314758770516591135802311025429227172106073057339165107921").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("2234398803815490360326379119253073021265347911436036657996997085101774002925").unwrap(),
					U256::from_dec_str("10214773739303944856355679056757071889148916152665035343340628435407174724717").unwrap(),
				],
				vec![
					U256::from_dec_str("21500623561133929663133180424780657403302270303706782392356903439186747104812").unwrap(),
					U256::from_dec_str("19328419280811286038333929820672543691407985372391054082759992875889788951230").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			14321691860995553260,
			7152862679273281751,
			12752615512303817990,
			1576113262537949146
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_10 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_30_input_10 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_10 Benchmarking failed",
				);
				panic!("nconstraints_30_input_10 Benchmarking failed");
			}
		}
	}
	nconstraints_30_input_100 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("3780487311936056381294564418675304364481973966802209296561160829797948105411").unwrap(),
				U256::from_dec_str("12392254879980640223142009257910424254771509444451393927739643876401726307795").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5169548117445173154024813698359426977000013771483410622256640533171985245380").unwrap(),
					U256::from_dec_str("11220449603083268927036844276912830985523128428499468942581840512730495047625").unwrap(),
				],
				vec![
					U256::from_dec_str("7617888807668014594076695037395251387521432502213194545531858961238344338860").unwrap(),
					U256::from_dec_str("3206448605631210983027663221710912427383659852717855953276353488187250079459").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("17260938596020642923034443094579912726381749511890996943502017876149586484217").unwrap(),
				U256::from_dec_str("973711408795313031099855712156150142641887502706075552254275664528776194028").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("4155898266631322214574856388057229131381603659142355428000239527171539910720").unwrap(),
				U256::from_dec_str("15919304600573806746067202721026223664629992455130946331020176989460510062648").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("10559477888637541108523299513483158185288656601390229631277602982302056830264").unwrap(),
					U256::from_dec_str("21518216923189214839507247102243744126930049565774641401859791113452178471842").unwrap(),
				],
				vec![
					U256::from_dec_str("20524972206762616004058259260290957844870248494446046806278424843369643489182").unwrap(),
					U256::from_dec_str("17430943082223119961631960086233630485654941075995072846024219998674392652581").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("4636362103930283526843587215716464395878300635435610009766312980459671398351").unwrap(),
					U256::from_dec_str("15713999866924132629927290621261290205960872899800820372400263081232019214301").unwrap(),
				],
				vec![
					U256::from_dec_str("274224283373606674211197218239218941312046485596075799302472837928822177016").unwrap(),
					U256::from_dec_str("10693491445395060496723678108216498586881120243971297703830999940561952271629").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("9540601502504552621404908678855872975596983734893914263055690675850649191273").unwrap(),
					U256::from_dec_str("10288168694112392204164781631687615725045294841966564539511156565183262528849").unwrap(),
				],
				vec![
					U256::from_dec_str("16504160099082952601826232467479510151840980336862791499394346450083636791102").unwrap(),
					U256::from_dec_str("9210167978306154509907086298616199856898787966613435045760659519455958969992").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("15812098394101742886061540987450559986219804469912805896076341688421241661621").unwrap(),
					U256::from_dec_str("11920211969663783445335622417572072568292106489672989284415235777753260953099").unwrap(),
				],
				vec![
					U256::from_dec_str("7061409000327933635430701054193661775345794014516696263402966093123651666364").unwrap(),
					U256::from_dec_str("17078047162776906089011285464212202278306676406084693061907121919624683442357").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			9831528597301135316,
			11053911270256492840,
			14337707091878126846,
			772611368070892958
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_100 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_30_input_100 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_100 Benchmarking failed",
				);
				panic!("nconstraints_30_input_100 Benchmarking failed");
			}
		}
	}
	nconstraints_30_input_1000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("21424271595387505713049534900676381136649432939050012296331964885140120988520").unwrap(),
				U256::from_dec_str("20488789316104689748479888121024139144355086386496471461763924485714534718686").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("8417699613540073682519718294821269056423071880647667978060348740795273915651").unwrap(),
					U256::from_dec_str("4036859279888161250878359321718658739242283134835030699023268489309754993434").unwrap(),
				],
				vec![
					U256::from_dec_str("16249800568747110450527363597274431407786919086057853194987169090454936230259").unwrap(),
					U256::from_dec_str("15005851458712291005106538565823270955208428726378749297397203143137382225457").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("21239990703864671714190525785674088715931473556140655107585637961585812134589").unwrap(),
				U256::from_dec_str("261928180985765917331919571442880380998408670702589931014636966043229919418").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("1657406292532781196449359079614579852690925607592278296574718289470803732053").unwrap(),
				U256::from_dec_str("18213557747942747074556848145237207574111244905157024361329330629173034829885").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5419856735820685908218319193906907396496344921642651266629131019868853843815").unwrap(),
					U256::from_dec_str("14007905369450574044915098075029863272475910046750101274280969769181784166430").unwrap(),
				],
				vec![
					U256::from_dec_str("182200131698860550444455761364918630551676350878779858078817264277300629723").unwrap(),
					U256::from_dec_str("6050431818377820601319883600432739448105157482353041283628740094609821419656").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("14463567879100083166257564330995533094334991709675023767975192973144798919122").unwrap(),
					U256::from_dec_str("6166287541473141918830331239835928582190744434356526822227225877844349692203").unwrap(),
				],
				vec![
					U256::from_dec_str("12512843744533735346513506961399336936527367720771340395643503621530703342395").unwrap(),
					U256::from_dec_str("8394818975982736201521422268641084055810886358791398607930311472872317386487").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("8757732756289429354419510422628896433407345473998310348115924113882282945326").unwrap(),
					U256::from_dec_str("1610185885725147927651201796201713521257480620743342969197368887869230657180").unwrap(),
				],
				vec![
					U256::from_dec_str("2834338214415807920713929739507915327763263469562351781747229949754640383374").unwrap(),
					U256::from_dec_str("20761263923134391843554423575366232120525285729512411038267095810712916924016").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("16585432354609922217222548226453277630464695131710476183479619760311567992522").unwrap(),
					U256::from_dec_str("5379258900528839934525784961038010961454863754298062530193947624450477797122").unwrap(),
				],
				vec![
					U256::from_dec_str("16825349369326282879106301528709807467071985531593215317852699692046311881677").unwrap(),
					U256::from_dec_str("10540450379202726868409867306854114915477889581096312866635875249145708179576").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			3008789406746563016,
			7061370202899273426,
			2191755323004339933,
			1005591455220622530
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_1000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_30_input_1000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_1000 Benchmarking failed",
				);
				panic!("nconstraints_30_input_1000 Benchmarking failed");
			}
		}
	}
	nconstraints_30_input_5000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("4760178114933307527976205229418881302833473087272893863010681924222436742436").unwrap(),
				U256::from_dec_str("6555056836023646163359279772724076765411780318148657506988760187885700674215").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("9540205596737585525189386078607957630969059503165883907845238817662495399607").unwrap(),
					U256::from_dec_str("21777913747079267536001051597001736802905779914683258171463923506869689358122").unwrap(),
				],
				vec![
					U256::from_dec_str("7652162643714058076103867092157642519402862581818067281609790687905833151647").unwrap(),
					U256::from_dec_str("5174136241284192352871549806655421810249317805219641765935618077180420544939").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("9633653417112713107386510132385608824105920168790226750481878454006412372444").unwrap(),
				U256::from_dec_str("5600932933208824148570351176368803282808856587672889614186338874655752173453").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("20563201722283471489193032980811350162387466733119686723832100493505332037220").unwrap(),
				U256::from_dec_str("458683337854426284881131074995322438618437423037306322290239745015633774839").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("802687519037697530569139206036966996588449135138601551566343295868421688400").unwrap(),
					U256::from_dec_str("10902489968444563758310974921240106861656913905085710523025258522577834664814").unwrap(),
				],
				vec![
					U256::from_dec_str("8163282908300075270434111132545381898266738500810094061899946027883982232155").unwrap(),
					U256::from_dec_str("2879234862354419255322548813414679750617024208122660306082351289172177493856").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("12946416399098615767859440028401625777970463338055590127763655113029764523456").unwrap(),
					U256::from_dec_str("19000556224902286061769815260124368973839417953399533885850283592096662743016").unwrap(),
				],
				vec![
					U256::from_dec_str("16883066945921282046110808795753929362007100542422795769422243319629113077450").unwrap(),
					U256::from_dec_str("3229017908221308993692674040951942449106283130433174678816420254626513367678").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("16459854676924169388774046538295013154707390397685062918225181412137603478923").unwrap(),
					U256::from_dec_str("20996869365345436816737319903210535785982053153529758285286719146777762981985").unwrap(),
				],
				vec![
					U256::from_dec_str("21446959969822179574296836241978980045198577120638747119902976563053247153220").unwrap(),
					U256::from_dec_str("4716297282146551186330252875195511360280503254452482577157100992033023717484").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("2127664998061023131080125201643658604092063304061707222326237054428506173927").unwrap(),
					U256::from_dec_str("13918137729545385379467184110597023571257121066683873930760549927318081992996").unwrap(),
				],
				vec![
					U256::from_dec_str("16560640526003408304721205785917894782070253870764339317137905014294472374327").unwrap(),
					U256::from_dec_str("19394535992291030838186763750684977819230291410826312937425829210188321257228").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			13212982172557049181,
			17043673098390055166,
			13971391527459105160,
			1729491887349651790,
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_5000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_30_input_5000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_5000 Benchmarking failed",
				);
				panic!("nconstraints_30_input_5000 Benchmarking failed");
			}
		}
	}
	nconstraints_30_input_10000  {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("20201588552006483041582128307591335561381558661958982754827324334159524114833").unwrap(),
				U256::from_dec_str("17768889616334897699714190449991059857597665579001510168911964966707846254013").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("831427914683469755258685737297441858923030844026909228595596228891513784401").unwrap(),
					U256::from_dec_str("17358733390631981546461856303504163975689822422160671163593908204299387350560").unwrap(),
				],
				vec![
					U256::from_dec_str("12673178383954759426828314411760863182427996098705680584931403917390222262168").unwrap(),
					U256::from_dec_str("12056092533572380482681874959737292916971452073896642753405050795241584080793").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("11086630979744216291689656728938462056245402184490217943698398792953833540899").unwrap(),
				U256::from_dec_str("6981043472993913602669753829495794560155248771335035866541870305230603679484").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("10088605449330278081588905225706913827985798761474642054412607299987851491561").unwrap(),
				U256::from_dec_str("12349112716571349951448807635426375878752472531266562453302627732729311140226").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("11234930657858483584603353214391341220392609835907149947974795594440445721655").unwrap(),
					U256::from_dec_str("1047093079337214465686469751128183347143657884406521365857439579470759333731").unwrap(),
				],
				vec![
					U256::from_dec_str("15864899975231343872664645828141071246029692141564279838230890638453916423742").unwrap(),
					U256::from_dec_str("12053277475368564348616897081746431255423835262329161057768339971384736640081").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("20662189442791195318296211376162761628558291902545582141646463139520777475283").unwrap(),
					U256::from_dec_str("9279465006661849499753241364514057268552316193158700659184124742215268752791").unwrap(),
				],
				vec![
					U256::from_dec_str("17033606864312833588672465768726096906720807717470346504048425104291392352898").unwrap(),
					U256::from_dec_str("15319242533246792039334613039385600689133248724151496600840914674340560979641").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("3303817333656562581428241644233610206796391323593261940899405821542387319372").unwrap(),
					U256::from_dec_str("12434486975702473910116656089923156849733601159782506519440013280731025013736").unwrap(),
				],
				vec![
					U256::from_dec_str("9561232329300105849651786447909752108987569955850131116304832039966768267007").unwrap(),
					U256::from_dec_str("18486557213071145739422364713271115450952939790314495487369777335095317193174").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("5864321703423906069527768678401445332135246771872328360194332677453860634867").unwrap(),
					U256::from_dec_str("3305620141441221125469209629518919979790470359050105194093502369856652644459").unwrap(),
				],
				vec![
					U256::from_dec_str("1810088252629429255118238329671475280643793553294287131634484491846973310510").unwrap(),
					U256::from_dec_str("20707970746663158481851347112547705553728126185220754421254503093218351233120").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			11033755416221607095,
			14172640425079373737,
			17268699570210893519,
			2451003838444787962,
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_10000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_30_input_10000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_10000 Benchmarking failed",
				);
				panic!("nconstraints_30_input_10000 Benchmarking failed");

			}
		}
	}
	nconstraints_30_input_90000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("7949275001087845673352113652992317737416671749823042856819742064958185730665").unwrap(),
				U256::from_dec_str("5891863541401068443747112095836743353338477825995666306081231182884954636005").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("12960565378826264234747222392568347150301360486418725511109506548813034322775").unwrap(),
					U256::from_dec_str("6924824876048701587380679111548259437770370772320848338373383587483809707515").unwrap(),
				],
				vec![
					U256::from_dec_str("21689747160304810742710652756166876177223750769696622871469266436845305438116").unwrap(),
					U256::from_dec_str("14167066926549192557125941689316913820167617731059401320567819691908586156039").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("19186797317064887099470636752510606772753033946198593325337771098325482447274").unwrap(),
				U256::from_dec_str("3350186543680262034371474397706130211690310055235028696466009478959025709845").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("6828794497030502754628523747507406719479438181845328406281207577196379759609").unwrap(),
				U256::from_dec_str("5561027388439947090438746111225212164455366280326670821700904471009784329896").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("1648517694167305339737563900225685665396551354808983529387892872646841040850").unwrap(),
					U256::from_dec_str("15406833006990307743521713564090315343374212223020060642984366610793788667116").unwrap(),
				],
				vec![
					U256::from_dec_str("14591594646846027404699868040454897990147675572067174007423843530180728597612").unwrap(),
					U256::from_dec_str("2940958434778941597191243007999700895445672570190450648227198352981565653913").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("7676142418543194845482248509044991512324196983308812731866772308356979871307").unwrap(),
					U256::from_dec_str("21511013842679919895947796523736214038726875927496967606596253311081756690725").unwrap(),
				],
				vec![
					U256::from_dec_str("4652378032872178978450279958290699982888059033676566152376150880744787185780").unwrap(),
					U256::from_dec_str("467504210088082002071026778234767679654059202832780062786965483300786044530").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("20744282071154891204049714936088748576694067059171789505913191933597423689555").unwrap(),
					U256::from_dec_str("12596866572762256583276410876169296807308160857887657727882625055041251553242").unwrap(),
				],
				vec![
					U256::from_dec_str("3588798969790877834321031026222278845206273301850566393834657489341158616535").unwrap(),
					U256::from_dec_str("19268326464935908053037454129640878232926568425004490855381855273084219557079").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("10928515506460596401961934744263657631726985113185854450754018401146226707446").unwrap(),
					U256::from_dec_str("1043421123157233618994554398894819122012315878788274412803269234825060696656").unwrap(),
				],
				vec![
					U256::from_dec_str("21802758012415709270812560952687831790545411568500639317690833457802568775035").unwrap(),
					U256::from_dec_str("8926874931677263335476856545765471099163606107894025045476086328360781102193").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			2553457387449696641,
			815105491620421980,
			17800546104409657428,
			2866567216183504159,
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_90000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_30_input_90000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_90000 Benchmarking failed",
				);
				panic!("nconstraints_30_input_90000 Benchmarking failed");
			}
		}
	}
	nconstraints_30_input_200000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("19903885251949679146607224162906437591286076549652044731739108226015926915860").unwrap(),
				U256::from_dec_str("11870922578907159231760438847634888925027596611865086487008358810346509550746").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("7572350741301793140585595336451242554319145556316993930579910816773197602180").unwrap(),
					U256::from_dec_str("16166630091911008518878343585755669995589761813810965163203815419554661335912").unwrap(),
				],
				vec![
					U256::from_dec_str("10479558631339350148140324543423297113164179377516410954282474094862685762111").unwrap(),
					U256::from_dec_str("1538599836393367534859442557225095989831507256094436416355361165563840234277").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("18566368172831132986640612657510108843544042128267080758083569019690510537842").unwrap(),
				U256::from_dec_str("14637144036179606956000163117625168358523245864386632910728645092984303842870").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("3388366987275268645608688601800038483480506575351380219459271970783548315310").unwrap(),
				U256::from_dec_str("10139982329791419477606385196757134153200700293861474061701578521738893563314").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("17293003127353596916845398615566872053529832484770184120881889494052646686398").unwrap(),
					U256::from_dec_str("3746742577904007471872553744363212404516454459540143078991207693255568088158").unwrap(),
				],
				vec![
					U256::from_dec_str("18996426015092705424501193523094438770040629737761363033585766320236602756452").unwrap(),
					U256::from_dec_str("5993252813486169444483155589566709251331252167093428959716521680127596688663").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("86221489590459042456285770827248107701675460052532768290239949641616327672").unwrap(),
					U256::from_dec_str("21618235778373592971185937996314200926578828337383413751502402964191761009384").unwrap(),
				],
				vec![
					U256::from_dec_str("10453895050472290645179709462208119639593757329666114205922088367970822668124").unwrap(),
					U256::from_dec_str("16924749634830145681809164670725933237166438445526123149706427927968513710893").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("20506347836984398194650179863264269358088067779451739307543701941138658771117").unwrap(),
					U256::from_dec_str("15256122081275324863091225387225218399888282841698960666742408225921079848693").unwrap(),
				],
				vec![
					U256::from_dec_str("7881380592415293900063452944037916715621668238658562838630198037249553436647").unwrap(),
					U256::from_dec_str("6003465274284900873747728448177380007802003937728590930467990944623586277089").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("5531662187275540864474040102019845108669876235687831703751327129559212127084").unwrap(),
					U256::from_dec_str("11455786949456856409363436892267690994602451302132959153447251790964376014822").unwrap(),
				],
				vec![
					U256::from_dec_str("17221728819314881853232886080533000381382340762620795831983019279828981481257").unwrap(),
					U256::from_dec_str("19341881794793455210912817199750586531241336493382870655971958667936340901503").unwrap(),
				]
			]
		);

		let valid_input: Vec<U256> = vec![u64s_to_u256(vec![
			17510768071209558285,
			11281672863963944712,
			17936011136389338927,
			2683423138755332222,
		])];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_200000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"nconstraints_30_input_200000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_200000 Benchmarking failed",
				);
				panic!("nconstraints_30_input_200000 Benchmarking failed");
			}
		}
	}

	sum_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("4479438672593572194843316486259284641920166054544576564310150763796220593742").unwrap(),
				U256::from_dec_str("5717857931841301866217201485541887040818093727574311517591849036289507799141").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("438102211713357984070931589322877643391719193099221891191956755149301376056").unwrap(),
					U256::from_dec_str("14696299438400225528910058565202329177181471318954607581306780544413371767738").unwrap(),
				],
				vec![
					U256::from_dec_str("18723112929967827488252389384208353884816003870032279933823625467146809072448").unwrap(),
					U256::from_dec_str("781225142569358922067919798403956593420009838230922093237868933021043854113").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("10481213903673023503381125372528754955378591812884098928519623211332171467098").unwrap(),
				U256::from_dec_str("15098195705682978185145469811146985310923026878303775073727891846327587112885").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("10600514820627226711889163536575359330497844540484876797299835164575096986123").unwrap(),
				U256::from_dec_str("8955505242614929800342716899005854501255692108538633212406728467939250778259").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("11358515176167160981846987587695931342878788452018419590778128561277894807040").unwrap(),
					U256::from_dec_str("7367100375568995524851051084377016949291621141008718892167611998460177966556").unwrap(),
				],
				vec![
					U256::from_dec_str("9986374126991936592889887298811366506288970021158383914729072242552712258999").unwrap(),
					U256::from_dec_str("5917047061876555575617149777207682047359711575786648519096749890796145313142").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("12470512550870605078154922234115021580073605821744497867586706681115325377951").unwrap(),
					U256::from_dec_str("3964373607930903502336465614571930627592350216968107805643078708599943392195").unwrap(),
				],
				vec![
					U256::from_dec_str("9054619931591683490625148848351137928205510101241639415537408964292455344707").unwrap(),
					U256::from_dec_str("14433710971468466044075036729227299898791551573181475688890434779549865103322").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("13008751738470482081156013535203222043379316308698376931322848604706786065862").unwrap(),
					U256::from_dec_str("3415558414191757405655644967374577939532374518318643852418890898323033638525").unwrap(),
				],
				vec![
					U256::from_dec_str("2104722493853714523712648261788727756558190509825976396834309269877977849365").unwrap(),
					U256::from_dec_str("6877732520157662584755946842350755740172704606349660337522459938993730313391").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("17691779713798847872985546462247707503645016076661816616388979822201176829753").unwrap(),
					U256::from_dec_str("14451231434311244057691431183198403713720423327382654964068207207079455187712").unwrap(),
				],
				vec![
					U256::from_dec_str("13620414910695185146987530463229819484652710069270717148541433045978322585419").unwrap(),
					U256::from_dec_str("10875872907505029426874795808985164080699672034676409959252328566889297105865").unwrap(),
				]
			]
		);

		let valid_input = vec![
			U256::from_dec_str("55").unwrap()
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_10 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sum_10 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_10 Benchmarking failed",
				);
				panic!("sum_10 Benchmarking failed");
			}
		}
	}
	sum_50 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("3114941071688532747454764921300681791980888248199392522675009157778549895455").unwrap(),
				U256::from_dec_str("6816680925124829082835883802586391663161606011222680818599486084577544527516").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("15037393792780247314317740287895838240941967969445981241792818111317913065010").unwrap(),
					U256::from_dec_str("3834028682995799272226496422556310257938795862629020590068471611902272764146").unwrap(),
				],
				vec![
					U256::from_dec_str("4023883034822047449129381158539093978377425715903419218696272130851521242085").unwrap(),
					U256::from_dec_str("13058906875228264286034925941047330220503025517973111955115072416984072943122").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("5344888056894889487007260648469199417556856955544355154291642477677322944786").unwrap(),
				U256::from_dec_str("19843194668352645797068691966948915569058668700830773246420232986019393210063").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("19257923683733118221652593587810195852599358412856281104090957905722976964370").unwrap(),
				U256::from_dec_str("19519729727735873999148754136012144616045305047716967685280834855818506563720").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("4418606700813566482216037680863387214037640478555893901955308344977554198718").unwrap(),
					U256::from_dec_str("18046767849922558936185002168253454007598159984067380537392936220227127228867").unwrap(),
				],
				vec![
					U256::from_dec_str("100725403545603844224319783153051757027683253433977048289134569498829183303").unwrap(),
					U256::from_dec_str("9785321212288652698655727476617710557399711491394256119637648908536562741634").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("7184324332054694882815451883063466258810341506246659357034202660488015984306").unwrap(),
					U256::from_dec_str("14398081398842334723334363407187040569418119441776629783632309768037771007327").unwrap(),
				],
				vec![
					U256::from_dec_str("17277213124622419955089087696435667358779233792945822269342368397066818801481").unwrap(),
					U256::from_dec_str("19811781065417531456401229702422997450338397507283920558741452740912313583930").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("1627724482578854283892192357407161928314687375966263116172485631707694669812").unwrap(),
					U256::from_dec_str("6839802000376483025048211234449253336394632788399066904541666906747650234382").unwrap(),
				],
				vec![
					U256::from_dec_str("17974577137954059751166162247265070307968439813726452160113109110701162631938").unwrap(),
					U256::from_dec_str("21289972529626386698860098501676961527414208445147370800462053148477683157991").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("20284064245383237908740484779775789109386762825985839657586769414934584661714").unwrap(),
					U256::from_dec_str("17363934481847560975462360104367791101351872886050159351690589725380050257409").unwrap(),
				],
				vec![
					U256::from_dec_str("3904403738619431633601997032597659843284744078768864151280168218762434423692").unwrap(),
					U256::from_dec_str("4413868989856264583489102855186788724178233679332360263809134874064797750675").unwrap(),
				]
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1275").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_50 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sum_50 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_50 Benchmarking failed",
				);
				panic!("sum_50 Benchmarking failed");
			}
		}
	}

	sum_5000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("2104134032972066019913571175532882184351637942839921828138208604817397790978").unwrap(),
				U256::from_dec_str("12625149522977248086293073767826715132905823679345516244316272790389621024575").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("18955718237898317556602416465218566057031982163829528123671300051574728240883").unwrap(),
					U256::from_dec_str("18075306515520608621953959878874546391551215874677138383864573851396832159644").unwrap(),
				],
				vec![
					U256::from_dec_str("3166476742865671627115278436666969186969644878053983393309132722947819232105").unwrap(),
					U256::from_dec_str("8051049247607109861344268872842958063551670273736053865524986958976361214062").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("4814291763265323888812051590287571407031542635142019809665068757723218701627").unwrap(),
				U256::from_dec_str("17930948070313125851452005455953009375012074818817251382455220462443054837203").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("19624232987744880202266197753309961915632611879850947888541508003701736644214").unwrap(),
				U256::from_dec_str("5019843555387010050200067558096501184079196549422645876015184268981691000261").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5150307298606577026396875555180704696345763655981810466632357002439129266461").unwrap(),
					U256::from_dec_str("3020656302335648031712857271727816386298675153130933709481311850060508289333").unwrap(),
				],
				vec![
					U256::from_dec_str("19547576195358452722114991449875886954598568010341500952076854329553571219720").unwrap(),
					U256::from_dec_str("3895331072082263654317901227968966808293407907751128382134753194859731114729").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("5792479463369352075979003547777370241697788651005116088867026457677720533395").unwrap(),
					U256::from_dec_str("16882159064341957118950974052606760693554763076237663579029547802558086431378").unwrap(),
				],
				vec![
					U256::from_dec_str("15201299876801474331768851307298179430198466709727112330042020251492139993850").unwrap(),
					U256::from_dec_str("893863620414771464835789333501093176356026692988850717670551586498757709875").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("165136411129701665870237237981894748637834826722462614973940945603783242464").unwrap(),
					U256::from_dec_str("10766657827945181878791518932726777890925519579254084496220957371082172349194").unwrap(),
				],
				vec![
					U256::from_dec_str("1813485593016293809685571822392302147347136682526622353828238069063634409991").unwrap(),
					U256::from_dec_str("12730118542350484957903782401323352413406115135420607527656226977122246298735").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("19450458898299554306247087083239008884256452312356497246598109368644839627523").unwrap(),
					U256::from_dec_str("123992241310851816418828784530061727329258706748227789362318537744292975333").unwrap(),
				],
				vec![
					U256::from_dec_str("1650700423026830428199086201346242616543834285406293983963791012821269466913").unwrap(),
					U256::from_dec_str("3152920600939501404817148774751227981988970987313622659392580888754616114976").unwrap(),
				]
			]
		);

		let valid_input = vec![
			U256::from_dec_str("12502500").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_5000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sum_5000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_5000 Benchmarking failed",
				);
				panic!("sum_5000 Benchmarking failed");
			}
		}
	}

	sum_50000 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("10521560297681522924956535552101557954055377883537271587209095331672512231293").unwrap(),
				U256::from_dec_str("3686654512637749683446869119197341068039068905923579159423036344564025864377").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("4890395955490088627121586512165392468998481877071173820388882318632254821251").unwrap(),
					U256::from_dec_str("9885243823600363166628220433820250724586324090253611358758770212725684204135").unwrap(),
				],
				vec![
					U256::from_dec_str("9651185291553128823215307092565172143818838202635271809942597163630971976482").unwrap(),
					U256::from_dec_str("7457264071531012293581639749136580536924886204589417402009399054127419592993").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("2627345354605748142976160304028628661480987375506843588233396593542954599122").unwrap(),
				U256::from_dec_str("8681729968580301984067665769277363721394455178806371130831646417737890294377").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("7421321362170719195081023585470135389432297716397232604489422086989398701183").unwrap(),
				U256::from_dec_str("4072026971341416803293623026670767879574289631340879941991921870784524426920").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("2973474839676012411019522612887674225146102238495561333683790233803898072056").unwrap(),
					U256::from_dec_str("4756088420398356546587372215087817373004751038308642279979561650652947628544").unwrap(),
				],
				vec![
					U256::from_dec_str("18134483811751652906456539288278265005968212485396038963550793901997203165973").unwrap(),
					U256::from_dec_str("19733531274857979315351276512779996430041767061694237978460875113100798536651").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("1470135418689977789083190292952479192678178451735632796795673595352524120366").unwrap(),
					U256::from_dec_str("5512330648930976997879473235916847191061423400686751865867764833860608466852").unwrap(),
				],
				vec![
					U256::from_dec_str("8727385532414213805687214017949229267547242769169729533774216518510791901252").unwrap(),
					U256::from_dec_str("6002526157648728322271067956991922780742552704883496238917574496345518669191").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("20953112965721952332699311075873353918826966291896395544882354080425788226212").unwrap(),
					U256::from_dec_str("9244986330211753947979669564454335484807935852348100319901614192766392809174").unwrap(),
				],
				vec![
					U256::from_dec_str("14633091132354386243825187195737842488061561784614981035737595152471541943810").unwrap(),
					U256::from_dec_str("18226447293937452659919522397971158104569327938800893595903771586125572293911").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("12814717307556312189651039282215441805578074271611267777533714458773278844960").unwrap(),
					U256::from_dec_str("7186820657304904476047212967487229874520184333187156276378594443001564484591").unwrap(),
				],
				vec![
					U256::from_dec_str("555198418426478766253895845611805598077310216839822834188225932135183360314").unwrap(),
					U256::from_dec_str("13253206399154373951823381980703846825450333469684895600618793136364702673992").unwrap(),
				]
			]
		);

		let valid_input = vec![U256::from_dec_str("1250025000").unwrap()];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		// log::info!(
		// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
		// 	"Benchmarking call end.",
		// );
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_50000 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sum_50000 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_50000 Benchmarking failed",
				);
				panic!("sum_50000 Benchmarking failed");
			}
		}
	}

	sumout_1 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("9938350861216112811054231203277972699126921197659464974935544809385001807185").unwrap(),
				U256::from_dec_str("20754348601909610501217030084772328592382611181274973942452539895656099027177").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5515305015868084342629690395946038238674727665234420855311483453081429329162").unwrap(),
					U256::from_dec_str("17446537483455495191309223490656602496508158001423723415129290719905124468748").unwrap(),
				],
				vec![
					U256::from_dec_str("3930386611358464774146562361387598962026666990617968852149397208747098046424").unwrap(),
					U256::from_dec_str("12749664200607251956437072816920531804207920954797765500171403273087430585686").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("19560759376029246774770917762129576352981383276741459039515891857991305382724").unwrap(),
				U256::from_dec_str("15734770292021565005778827149409535166727027683377260372189150049608646364563").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("21733217796115117340664094616124062556432735361619337933379922909843902055515").unwrap(),
				U256::from_dec_str("11458528445138916020920755047732691641150418138517915174130099425004410419388").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("1528892777992101851174077814499016728285240514808795953470350987666406934834").unwrap(),
					U256::from_dec_str("15719121553978478977205161897674358216198301946293493012613276233509561742855").unwrap(),
				],
				vec![
					U256::from_dec_str("11348304938307062180350959111763404727439270384731375816552220511782794870928").unwrap(),
					U256::from_dec_str("7425062298693286516955806409965528686336065862277124413414678440080583191331").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("2804875110900945674713475978302264950571804196556173252173183478206581329693").unwrap(),
					U256::from_dec_str("21248149018328674674239251862093725699159113249722538926825249085881448126937").unwrap(),
				],
				vec![
					U256::from_dec_str("11937905968906797411243920288058802112704397972843813290927323952904789450154").unwrap(),
					U256::from_dec_str("11583994651687877961370144800492703053197084517566628648623282684591065755611").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("6948345332843670535874384951250604863187455917521854956702988343215236877699").unwrap(),
					U256::from_dec_str("7592227853391758702072244811620141194222566425322898305556974168308138901071").unwrap(),
				],
				vec![
					U256::from_dec_str("20320075749881446343183062003450485857646019655198241442555760885804303255899").unwrap(),
					U256::from_dec_str("16287761203840607067547903820157586575039034353110142941200011092583966755993").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("5903727580143985844443655542650980356290410430996215778341658075199847464323").unwrap(),
					U256::from_dec_str("11996884259430469116666694100434422195076194078215478638845899818616578458076").unwrap(),
				],
				vec![
					U256::from_dec_str("7085337881114239321879072118312865301523763704430159554418691955396215633039").unwrap(),
					U256::from_dec_str("1480130379115126141544726883503597589909960712327226527274861716229740605644").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_1 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_1 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_1 Benchmarking failed",
				);
				panic!("sumout_1 Benchmarking failed");
			}
		}
	}

	sumout_3 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("16886990047698791021397211555946902052303558083480918437568754609520399637420").unwrap(),
				U256::from_dec_str("2158666036923715472550204975056204244361247562519274259158726699413361975377").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("21825895835348066771949343990501317097894843683430512250115671134066620251631").unwrap(),
					U256::from_dec_str("4466621631021858506469772043982203005266775866614490870641688212518524595581").unwrap(),
				],
				vec![
					U256::from_dec_str("4176035282152177060631306859647233493053656458958851103499068538408191035948").unwrap(),
					U256::from_dec_str("11478054618053509184726472858088324642813538303220488990534337782294436141900").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("2783215660066684652951960961708568160262083911582630687913502540009978493094").unwrap(),
				U256::from_dec_str("12293216536420324710913866412884718473844766370712205111375513783367059491745").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("5173033808064965357658509659910492294930152276344605525901535639502011399348").unwrap(),
				U256::from_dec_str("17068737078093325914601548755287000154671449528138414861362494877844369116679").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("18546422218544016845270499276237522358419081403760538321534730027231631447721").unwrap(),
					U256::from_dec_str("7810445849810537796619946419860277733990747614871399330031502459697394450024").unwrap(),
				],
				vec![
					U256::from_dec_str("1170639424349054655683720291779988770761452909673727094673081217226173989816").unwrap(),
					U256::from_dec_str("9412518718720962495243722595927760663661173673000956950863625218254861399884").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("2200736122860885030116625882728423481103104963926248602047366582063573350368").unwrap(),
					U256::from_dec_str("10144419795729765523122504674541920919692895267022159856625676910310683074572").unwrap(),
				],
				vec![
					U256::from_dec_str("13390729254545550118668755621583864033913527349600040291864450051082409115353").unwrap(),
					U256::from_dec_str("10049685629671638387934361525360217384621934210702130547016579632581199513700").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("19667103121024340857751329572172914473199256004525176516911283862315503618923").unwrap(),
					U256::from_dec_str("17096246661332763117514187398699870735557867602087681563046129321059799057215").unwrap(),
				],
				vec![
					U256::from_dec_str("5747946728261847324885781741547355414535058240068054018513487219209938622249").unwrap(),
					U256::from_dec_str("20967849068573472545470716505852386189926392787722866540330567432920833499864").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("10845237408084729771758679531905348867405129151866839173812886431855390540097").unwrap(),
					U256::from_dec_str("1703707016089191110685983076484608595856556399796329016216676789467749613552").unwrap(),
				],
				vec![
					U256::from_dec_str("6110149777882922151130458750708717975247447215574560066901560999540213459036").unwrap(),
					U256::from_dec_str("2604335473336229760265035630255446069244978528812036061612414998445480891974").unwrap(),
				],
				vec![
					U256::from_dec_str("17335232688503638164872257108646124236545159761624108797282027627835697028104").unwrap(),
					U256::from_dec_str("8949820120400561503263628231115650124871194346512209521952934432568756809020").unwrap(),
				],
				vec![
					U256::from_dec_str("17472945225672873683007250170922636087596020485883147904995489212967632436245").unwrap(),
					U256::from_dec_str("3061009963934218559141330300326228190658110279298003565222855302662460452130").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_3 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_3 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_3 Benchmarking failed",
				);
				panic!("sumout_3 Benchmarking failed");
			}
		}
	}

	sumout_5 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("10843709274069701798511921250280301584693334300670909217788413041083651717163").unwrap(),
				U256::from_dec_str("16984929187729614385413559875801633684939267739432542404983191429879545973363").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("734148988538591964874068908106884716790741858016853636848093036568317136975").unwrap(),
					U256::from_dec_str("19009101126100851400074367739886512661512111552427522714684639750197874542269").unwrap(),
				],
				vec![
					U256::from_dec_str("17592060285412708307279524116763043954397813345633635123760549183251104162936").unwrap(),
					U256::from_dec_str("13889506181636214857793531048122114805475801077448416250517876148460169211100").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("14232563060686721866158930720630875371569684145608628878932337428224050961095").unwrap(),
				U256::from_dec_str("1153379304505972153074194911346366079704012789387132395115115570476601062069").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("14602929204623575899704572853140632308492709111903618875822489562329338716600").unwrap(),
				U256::from_dec_str("1689586738876602514816831803381017781452340558398178586457594640291347324251").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5472399972818221760148387617622606184408186046073893949062037156069912958420").unwrap(),
					U256::from_dec_str("15088758066943253549000189946806726744988559838853537684885055598950377383100").unwrap(),
				],
				vec![
					U256::from_dec_str("20965140394004681264140187314264751937007442823435473099253896953347314157448").unwrap(),
					U256::from_dec_str("6060690771018416217128945435034565801294536828990641996243103214693154087328").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("21263140729235433474630918705483709183043987469144755035836936799756516942174").unwrap(),
					U256::from_dec_str("15454285754854480671888484669874181801887589102853817499490458030152705038393").unwrap(),
				],
				vec![
					U256::from_dec_str("131865583299313362102439649370460452157962165364517843475166565097979328681").unwrap(),
					U256::from_dec_str("3031199517487834223134891339647304484515505296778183327653617685889942730653").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("3468906430674013119693695928268572867728233724074466649670021386689854137425").unwrap(),
					U256::from_dec_str("7152580803122005503226572138561796511745831544828490575855116764117124258005").unwrap(),
				],
				vec![
					U256::from_dec_str("11838997786679875543165785538783421655549584031892121869365192734986828433584").unwrap(),
					U256::from_dec_str("9326037833969675641798059567930300590354374810107157670596201469638516559516").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("18012654367772022563720142058930749289295704020857186339096955550145993927017").unwrap(),
					U256::from_dec_str("1820834164495870618274456212936566261697357111580171595080158790515959828160").unwrap(),
				],
				vec![
					U256::from_dec_str("14564921752272533367499159134818448151761069174539749712526942006044013367624").unwrap(),
					U256::from_dec_str("21424771264673466693648639329731655135896471263362765164636863696917694576743").unwrap(),
				],
				vec![
					U256::from_dec_str("475727869657896018962474075772680190259478188474158088185455044188053740464").unwrap(),
					U256::from_dec_str("14410774738558525043590957367040603027587866740036883757867616381286145538306").unwrap(),
				],
				vec![
					U256::from_dec_str("11122269591667799379645221047239470153918191911025387915741500657620168268073").unwrap(),
					U256::from_dec_str("17658889114158552675967451148822309152599407578222142601435191444872800075124").unwrap(),
				],
				vec![
					U256::from_dec_str("10247277070171001623223120619585458877815606242596599730030571392627204008518").unwrap(),
					U256::from_dec_str("8479449182628070681609638314620998919468112761035511418595900534618900240479").unwrap(),
				],
				vec![
					U256::from_dec_str("1962015739865113713712043429590435981426176192245833450140313755224798839430").unwrap(),
					U256::from_dec_str("16205122197955925140750583202721983405250616309734059872813415529671902218508").unwrap(),
				]
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap()
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_5 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_5 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_5 Benchmarking failed",
				);
				panic!("sumout_5 Benchmarking failed");
			}
		}
	}

	sumout_8 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("20881676737166493582717361049990209142091151560277146607338080001356130551563").unwrap(),
				U256::from_dec_str("18685148182132563281318375875896798947216249273522126721154367752167069271966").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("19805017985803942911996819094597598814225347516143362171828062466401419556425").unwrap(),
					U256::from_dec_str("6454257483855613540563513163965581345545044507035534598690370868179178950555").unwrap(),
				],
				vec![
					U256::from_dec_str("6029759526221488701419339807738355794494802551955780707582495359517196927109").unwrap(),
					U256::from_dec_str("14267537981552092649796532473776382038697314253728184857929706479499924660588").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("9594194445262469071793565488528689703162142686015495470197658864675667005764").unwrap(),
				U256::from_dec_str("12806173576837505234722625501460681990985569361840943288163429373057791635174").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("16843199197487561646169134476888510727371602451967711834418709891625862192118").unwrap(),
				U256::from_dec_str("4982834246914059407767022322452524619121175829432542760151885667226965908175").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("13537923638822567297513803074712167738585661835781036936983516715534090948855").unwrap(),
					U256::from_dec_str("11214527989489337965037448828413554959537012329709327451546605605029975329833").unwrap(),
				],
				vec![
					U256::from_dec_str("4462141743449864304650718313134032562107623852111180981714224584326923063811").unwrap(),
					U256::from_dec_str("8593795430336743569036551388690578138795757924676449205217387344751477578392").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("17309166272661014787434443764307400135052663428379855697446417017021332297530").unwrap(),
					U256::from_dec_str("5496023192794917920401339916944184963784198502465903489239967703087855023608").unwrap(),
				],
				vec![
					U256::from_dec_str("18916827587111283766401091943196651682285539340427490406407460155259456790394").unwrap(),
					U256::from_dec_str("12299438544618647185398496571464375186770593791424093069019355848143426272476").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("17783472651095992230194984959634523916458279184006542713077780177849417194806").unwrap(),
					U256::from_dec_str("4836569650671318166980489785623247713000662692959335424089941395104953246823").unwrap(),
				],
				vec![
					U256::from_dec_str("5208397036236600320774363534257475251244044365748886930563031073914567820528").unwrap(),
					U256::from_dec_str("19055050817214474907565862351314552256205984947772618507442964780728061953573").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("4958353703564853797480558824501374493866572548677008630806682009679433646955").unwrap(),
					U256::from_dec_str("11291693312464961842059331703592439948750550091724954216017767042000996236264").unwrap(),
				],
				vec![
					U256::from_dec_str("12498275799103181792627863878557495829009523362285107656610654372282690766119").unwrap(),
					U256::from_dec_str("4660953423300268019590006016093722402476152146326204206929515385340767022285").unwrap(),
				],
				vec![
					U256::from_dec_str("7138500762518060350238754675766385436116813573098550938636446616835446824939").unwrap(),
					U256::from_dec_str("10593558873947627445673570470975584735484733850711765231108539595431195202419").unwrap(),
				],
				vec![
					U256::from_dec_str("5128578074800121244687450741397033494076458406794772068903912438713735124312").unwrap(),
					U256::from_dec_str("12739298773375695368884515558611087780028515164769718103051887568287014868554").unwrap(),
				],
				vec![
					U256::from_dec_str("14208333732881243428572299578791263825282606806313226027225601452663343234467").unwrap(),
					U256::from_dec_str("11697572475523427738804655258597612653697756472591022126910520054756212281641").unwrap(),
				],
				vec![
					U256::from_dec_str("12947417320200513077005132775929421525296222658070514082956015435086603716201").unwrap(),
					U256::from_dec_str("10603870931188623347190243514602488220386876361489556318755549006254763084062").unwrap(),
				],
				vec![
					U256::from_dec_str("16387770851924831042551337157942839466998720794986734312705189727537868513301").unwrap(),
					U256::from_dec_str("21530634239892293824934800960052121252877550002298146977275883229302441879110").unwrap(),
				],
				vec![
					U256::from_dec_str("13802739621009348580166301250636211668066199382162533891337095173708507625183").unwrap(),
					U256::from_dec_str("7390549673741696710038465132307739993957511234849566067102331114963870977526").unwrap(),
				],
				vec![
					U256::from_dec_str("7619444939038530089326050845035495508541658656202842508270208357152287148678").unwrap(),
					U256::from_dec_str("6500310581417507359356071153065385786035787026399006593758019508960807973327").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("28").unwrap(),
			U256::from_dec_str("36").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_8 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_8 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_8 Benchmarking failed",
				);
				panic!("sumout_8 Benchmarking failed");
			}
		}
	}

	sumout_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("8095347977558970795244894306973905848904001838702896366752039759486143590655").unwrap(),
				U256::from_dec_str("11389786227461730181613149052445453595828636283243249470864002334968553081173").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("21708613222549995537668812978970219933649440225134255022725303703626994194813").unwrap(),
					U256::from_dec_str("239188211415377402375320392238806616016565549618403346265681937965053169431").unwrap(),
				],
				vec![
					U256::from_dec_str("3239905157092125823769350435491033426008439939030717819312650649959874173134").unwrap(),
					U256::from_dec_str("12052232966415578751823016823727256388291367828304994281169873551977365031626").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("897402773553083836675111568099716002998450791951489272035491280385895453675").unwrap(),
				U256::from_dec_str("4302991846298387183398271555952452477837006359717803162778715536752182672022").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("1008042868469261437465824462555835872984405918513720852248805969522549317398").unwrap(),
				U256::from_dec_str("17948956462462396838284127242383169953057095846419418150844950289307977478505").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("8551104548742012199265803926546530567117656701099183084826067603858509033959").unwrap(),
					U256::from_dec_str("18033160402793654659203629055551676044915660275765706303943849950983851388359").unwrap(),
				],
				vec![
					U256::from_dec_str("18666245592989525932086991882662677749245174145376158182567864777290703859953").unwrap(),
					U256::from_dec_str("12739445590586817042423266191782746191509898418710009323864773881753277149266").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("2331829126412868358771801117979852694196692966771691214227043519819366059412").unwrap(),
					U256::from_dec_str("19888979373549023610391803146710987007008930621488156737270769648673859621478").unwrap(),
				],
				vec![
					U256::from_dec_str("2453673301402362948623253085750603643389422628968377872756565691580988083845").unwrap(),
					U256::from_dec_str("9663445074525460783023482351534650548924461390941888390004047901572505652613").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("16927652518569561664756655584955863290280627934729335681190762313953209291004").unwrap(),
					U256::from_dec_str("4932558367420707526402578793947314151706672793656462574202517097413681336095").unwrap(),
				],
				vec![
					U256::from_dec_str("2634665158965720057724065638899982698588247847852682145611641155204230491999").unwrap(),
					U256::from_dec_str("9690377563892165618821474298831099678985125870872292050827811231043721368626").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("15646752321283466165334982599725570177394246403813099805195136296761241527233").unwrap(),
					U256::from_dec_str("2673660939947951312370015187331632544726348142660533215912073160631461110194").unwrap(),
				],
				vec![
					U256::from_dec_str("8795725431636628672881798189389564709124948886181390178455547202534111009338").unwrap(),
					U256::from_dec_str("4110538773071852744620136310168055865424844624884280749230423420272832655948").unwrap(),
				],
				vec![
					U256::from_dec_str("1831722434614555360769800136025735307898058715006643917013942103396575104215").unwrap(),
					U256::from_dec_str("19989387745164304450269343019866869726210315559728705694008015771363654418682").unwrap(),
				],
				vec![
					U256::from_dec_str("10862696719365997074854505211779324198035920467781370644724921446857917651996").unwrap(),
					U256::from_dec_str("9116798754108731679801642195848458981493300338319414129478047681468689383121").unwrap(),
				],
				vec![
					U256::from_dec_str("16768252193135174475066659836349285218577351670934937518606124353068436732280").unwrap(),
					U256::from_dec_str("13373738945795732208212519161819821211235233264106510550098917898260605099707").unwrap(),
				],
				vec![
					U256::from_dec_str("542836645169683970941273656817512961230019569896206162978989160115050828365").unwrap(),
					U256::from_dec_str("3342534425975518224378168738779764102272719874861318177064435130732411210368").unwrap(),
				],
				vec![
					U256::from_dec_str("1770766751918227923442808905725672958134690651505108229474581836615986300515").unwrap(),
					U256::from_dec_str("21053352455656543786046861160862658391653611956434282730527309466510916590010").unwrap(),
				],
				vec![
					U256::from_dec_str("20880233474430230226643725074209261213944122688335980068588310045451961241696").unwrap(),
					U256::from_dec_str("8791463445716605276609109968668430420344826553389688804713588795902347288143").unwrap(),
				],
				vec![
					U256::from_dec_str("18577375943106027695484211293909818336439163081296645089367052818189591231338").unwrap(),
					U256::from_dec_str("4321732139643796718169445123275382387645361160952863834396437890748602330532").unwrap(),
				],
				vec![
					U256::from_dec_str("10314948068545990837002165860394752996164087839142117273228424101259185001002").unwrap(),
					U256::from_dec_str("1219317476656503163020791040029674985228288088100785556071334850164078598664").unwrap(),
				],
				vec![
					U256::from_dec_str("20369783784101349031638736538393623333643160768611915809859932035797922480026").unwrap(),
					U256::from_dec_str("2022155339791641519218630090672540669148716363586469003730495586402216125347").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("28").unwrap(),
			U256::from_dec_str("36").unwrap(),
			U256::from_dec_str("45").unwrap(),
			U256::from_dec_str("55").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_10 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_10 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_10 Benchmarking failed",
				);
				panic!("sumout_10 Benchmarking failed");
			}
		}
	}

	sumout_12 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("10678216327021239791495061437747847264659254646405410835995870885562189659878").unwrap(),
				U256::from_dec_str("8761704370667248778922819575959420459857647618994282299958428953088561107560").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("8321848050333625644069000415252127116589656568901903629556956574360865016809").unwrap(),
					U256::from_dec_str("8384658520212855102478260838149819693091725582498835147548352582021949594963").unwrap(),
				],
				vec![
					U256::from_dec_str("3770359937661059505252608927767095077492661514400658547012520648582203048408").unwrap(),
					U256::from_dec_str("3529784532113968623662330699908137485954980022129653872444397209141826251473").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("4705370747374071844930946489229229872881908444966715664904386641107882480824").unwrap(),
				U256::from_dec_str("19986445778680556370923606923016458353128173727636088941150671422181521841587").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("10908812779208764458926225885786000177375281860565170285671641496279275861828").unwrap(),
				U256::from_dec_str("9799857308176907495837560301836136037191379657982632569243527710538263747953").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("8875574684981713823643201920386676384608102436522842095334134617286263764542").unwrap(),
					U256::from_dec_str("6479740794118221987496646366125424954976260650207607249416083588160064029713").unwrap(),
				],
				vec![
					U256::from_dec_str("2000063549109574433659751301145359407962925053546745695255853892415998211427").unwrap(),
					U256::from_dec_str("8978561107432458649913523321794599249451491250594440475529904908089104723622").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("300802291087457914851925428899689384077331048328269093618223108664964823570").unwrap(),
					U256::from_dec_str("8865288388013568285490817864856410066558523665602904501832397747932479098531").unwrap(),
				],
				vec![
					U256::from_dec_str("19869995418496215898062302965642426582779874214462186317706024702601899014194").unwrap(),
					U256::from_dec_str("13390306745571857664751285542582216134343088779623754549999133942932770241738").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("4865530596920678143514720246031070564475186170621062271332162290940892835646").unwrap(),
					U256::from_dec_str("425558782916879613747479872909542286212413009319811627068712669719204459009").unwrap(),
				],
				vec![
					U256::from_dec_str("14280364779325396808134659809671863728861497766987810166986747587937753155420").unwrap(),
					U256::from_dec_str("10157821355317239779823773394681666569085014825500396192845621348313614416302").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("5077257330623675885939934768370950104582038333816470525140085934362181966137").unwrap(),
					U256::from_dec_str("21590736165200670756412215987782110683410511192188292195822616418371654753741").unwrap(),
				],
				vec![
					U256::from_dec_str("18280467089374114886419816344796237013631762412466882786964276277982883666042").unwrap(),
					U256::from_dec_str("11485344019443762604807444303195364775881039088908610566033356938937497764373").unwrap(),
				],
				vec![
					U256::from_dec_str("1751181504701297498660939287755176392070958106184310143186418668952529701336").unwrap(),
					U256::from_dec_str("18736932220693579046179817485414749648152355566594839961039608036135808206151").unwrap(),
				],
				vec![
					U256::from_dec_str("21312152220282516002927156143098977719015192971046736078567907261902527789628").unwrap(),
					U256::from_dec_str("10860773398015071182683502891538197212844225511698561100014964817267559412040").unwrap(),
				],
				vec![
					U256::from_dec_str("19213317000677932000832764919824843611437100907614316132834763294564357679030").unwrap(),
					U256::from_dec_str("13111222029963575410995049632780974434271987081267940693386298389237806957098").unwrap(),
				],
				vec![
					U256::from_dec_str("3443742642228361401389419381310437614798303534754415153009098249760587588145").unwrap(),
					U256::from_dec_str("7744789485747118631461059019849731351689596466532055389408736826440805536735").unwrap(),
				],
				vec![
					U256::from_dec_str("7435039988569923075244353313547948053391148527555628591292741199024066970135").unwrap(),
					U256::from_dec_str("11046311630463377170355961221617143843117652341179127685539086936254925519686").unwrap(),
				],
				vec![
					U256::from_dec_str("15297665691078029562294174705734310900359658439705626350781557084124323125519").unwrap(),
					U256::from_dec_str("1364806105271580183714977095246233959855350281838839613346051539860853034390").unwrap(),
				],
				vec![
					U256::from_dec_str("16182913767360895651204241660486317958300561196841976764409472315424042489856").unwrap(),
					U256::from_dec_str("16339286891979828690592962765027450964989749947359765283073535710465615599250").unwrap(),
				],
				vec![
					U256::from_dec_str("21575807155821147451222063394016891163053568165439964049101545653944062898132").unwrap(),
					U256::from_dec_str("12371960356859909936672076205340736555848052976513199823807067061233917890308").unwrap(),
				],
				vec![
					U256::from_dec_str("8990730410639111775085854283714024130596570124701374392253236961070523441877").unwrap(),
					U256::from_dec_str("19255521192585577283930550804749005294045120462318692755332022063761472178542").unwrap(),
				],
				vec![
					U256::from_dec_str("13320084394492248060490191350301804238250075203939012239448020526455826338619").unwrap(),
					U256::from_dec_str("14257121049773316510902540165944619764221000278529377562530357678201148349783").unwrap(),
				],
				vec![
					U256::from_dec_str("10413866546283584888554122264730221359972621491764741530663968262596943342816").unwrap(),
					U256::from_dec_str("6517317702577541907505416253741115772092764156468351641670910298537704748423").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("28").unwrap(),
			U256::from_dec_str("36").unwrap(),
			U256::from_dec_str("45").unwrap(),
			U256::from_dec_str("55").unwrap(),
			U256::from_dec_str("66").unwrap(),
			U256::from_dec_str("78").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_12 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_12 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_12 Benchmarking failed",
				);
				panic!("sumout_12 Benchmarking failed");
			}
		}
	}

	sumout_15 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("21585823022431357826313674931621886543236834982289246714992743286376977868419").unwrap(),
				U256::from_dec_str("7586678945863927211661133741820075312545324139326102654404133129309244972746").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("17555814517186373100161784981688729927129395923544443054321966341581129282393").unwrap(),
					U256::from_dec_str("7282547604111412630488355620323254420866246961778488838010455131596810507996").unwrap(),
				],
				vec![
					U256::from_dec_str("20713205203394977430100275064206656162423873472847285768042185373389150331759").unwrap(),
					U256::from_dec_str("21594434550845851185653620887682118555994201709039016918315905233204134362395").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("9553527308364266671450755468661721124726046549095186915848897612832666669104").unwrap(),
				U256::from_dec_str("11201734029982017624737447275065436309338979180962200791637778279686032883214").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("3216230770444528088776158534242932327068728121815873071849223010607020335058").unwrap(),
				U256::from_dec_str("21035809065301821627526137231507537292096888122223808691041174759761119220267").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("3117446826555143230699140819891988283299523756237412166668770967385826469061").unwrap(),
					U256::from_dec_str("1106011150835632885010646871826845378592629967358666497747945481466306112631").unwrap(),
				],
				vec![
					U256::from_dec_str("2031803257794032494221338998687539109995321195789701372872082220755008700935").unwrap(),
					U256::from_dec_str("11356081879090717733075995723613325531791197494523954848147361327178076996398").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("18501235594532758685356617643293397665990404990535859450562994349740531359954").unwrap(),
					U256::from_dec_str("6064697307908680945151400145510311630157828610555420079802490329525473148112").unwrap(),
				],
				vec![
					U256::from_dec_str("7588288119916448725682829538677035363474240351868137059873143928863164784498").unwrap(),
					U256::from_dec_str("9274782715817123279177582291910200696665013934498334039985344489430227846656").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("1371652079761960603156000555240109860059987026059613009913618544836739152585").unwrap(),
					U256::from_dec_str("16438218313867099055033729216771189397229055480760839479410444015762186153011").unwrap(),
				],
				vec![
					U256::from_dec_str("21625074976737722082451218446149574583326591404236641360262678817113664694657").unwrap(),
					U256::from_dec_str("20796091416200429030650333330639080121294047317030102213022681141542267122695").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("2269623709920728903617646620096243620327949366550222548011870028242497613358").unwrap(),
					U256::from_dec_str("1721591364215678281087769539296348528917077738930021482596272432159152444992").unwrap(),
				],
				vec![
					U256::from_dec_str("4351290528334741701086467877839775176289883245427946235071571814919796918896").unwrap(),
					U256::from_dec_str("4623623087098123792032780719067218809631814301015875737326472476776737187664").unwrap(),
				],
				vec![
					U256::from_dec_str("10252611518525171358643906944080590081835551889067213413963176637715118491735").unwrap(),
					U256::from_dec_str("12808501284953465818004421681457302335036426618905271633262458529689968964126").unwrap(),
				],
				vec![
					U256::from_dec_str("10981451002832697773062138382859126275926702959532767987336745018160655315751").unwrap(),
					U256::from_dec_str("9013927030633677625778473876432702183373603209076101904574814352400708922379").unwrap(),
				],
				vec![
					U256::from_dec_str("20570333858794685470455854052142258100074980426834489402096708371481795292792").unwrap(),
					U256::from_dec_str("15956512164458091255235753566653194130138151604902408378082844639163963683640").unwrap(),
				],
				vec![
					U256::from_dec_str("1810920619373659655004366626576702325696132807184568159455273357389553192281").unwrap(),
					U256::from_dec_str("1003480095719919700122990934105236095705355474829745254732375849367589900011").unwrap(),
				],
				vec![
					U256::from_dec_str("13039615164171221481521921362114175385649000627043496255793789378858554035982").unwrap(),
					U256::from_dec_str("17510910511311660067327571445074639404525503060298062838447037718941394979779").unwrap(),
				],
				vec![
					U256::from_dec_str("14165329950765673430990332163153038866908713915209946170837286178877146886660").unwrap(),
					U256::from_dec_str("20793339422175421707121785907356839672588761368373996623737044948407075380863").unwrap(),
				],
				vec![
					U256::from_dec_str("3883256103961824093488903468324246690649140067919691822154370105123302528628").unwrap(),
					U256::from_dec_str("7561475158831403196993480443820632892645092082726176558327718750568963291037").unwrap(),
				],
				vec![
					U256::from_dec_str("409664433526832013116877287300401090775361274457633011032558918365352310607").unwrap(),
					U256::from_dec_str("17915319827370112847684027302775847360731246969152629190102908994479804002536").unwrap(),
				],
				vec![
					U256::from_dec_str("8783158326340533223560056224761519419016551580825409101025477639190392573598").unwrap(),
					U256::from_dec_str("13707722239341829918696323583233367883921076402405851150543609078213603790929").unwrap(),
				],
				vec![
					U256::from_dec_str("20473370039043622683429816021606510529102905509776081241485577312164955536715").unwrap(),
					U256::from_dec_str("10758908262111334689282452504541297068601729063936562246389590828664826425253").unwrap(),
				],
				vec![
					U256::from_dec_str("1475500216206520471930942390519271535125544856677111800223296044790407084884").unwrap(),
					U256::from_dec_str("11779748037669787286041766573080352957811656930851243042004933736078318402770").unwrap(),
				],
				vec![
					U256::from_dec_str("13598685577793421380004205729308386952260129613827073513489648865810711589158").unwrap(),
					U256::from_dec_str("8828420191868575584209720851114347878775413131164313846108918081819270797245").unwrap(),
				],
				vec![
					U256::from_dec_str("5841283600010020706817581126281272248796034770291297306880281256297906914129").unwrap(),
					U256::from_dec_str("19573442695439575558179849102246406448222304178063434531164484895634034938076").unwrap(),
				],
				vec![
					U256::from_dec_str("12732593393130810020796334300053525912829502041711447313821913815961692050846").unwrap(),
					U256::from_dec_str("2396355716633209699405102047603034959746626203843654407433538259302908360443").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("28").unwrap(),
			U256::from_dec_str("36").unwrap(),
			U256::from_dec_str("45").unwrap(),
			U256::from_dec_str("55").unwrap(),
			U256::from_dec_str("66").unwrap(),
			U256::from_dec_str("78").unwrap(),
			U256::from_dec_str("91").unwrap(),
			U256::from_dec_str("105").unwrap(),
			U256::from_dec_str("120").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_10 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_15 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_15 Benchmarking failed",
				);
				panic!("sumout_15 Benchmarking failed");
			}
		}
	}

	sumout_18 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("12430633769581102247540296520133403875606526791119053988338140734241680888772").unwrap(),
				U256::from_dec_str("18442332250316996812297811822409136992315114037253390339439613098875601509891").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("8769812720201850845212552423116339332017173636860658639095486287960968364628").unwrap(),
					U256::from_dec_str("15679609225338281006396442534228723562221249428725922637917135773932321194569").unwrap(),
				],
				vec![
					U256::from_dec_str("11736822938524029545900531415301990557371724695717091442770706694108764792897").unwrap(),
					U256::from_dec_str("14276131700804161157283912487255205591674223192147331953987891031341492138831").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("13829860464295019021916660157613562204483787530756857271489430053870068539213").unwrap(),
				U256::from_dec_str("2635154643612779766721540915999489277398619320264472919611172655006207593243").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("11478354889186703145064785718789051075161412405402409375958203264983577174612").unwrap(),
				U256::from_dec_str("2051932981799192988614815520842778215358192513999820653139553771063993577502").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("19202672763976837199930038746574488489616398769933191173401842283930856630824").unwrap(),
					U256::from_dec_str("9894585412193132052039710787371205702047848152467990282623721538452088473643").unwrap(),
				],
				vec![
					U256::from_dec_str("11087012751377729871020942087037603184858545608921665053341734432443613964736").unwrap(),
					U256::from_dec_str("19920592886298080734865733588618209252746933681013350273253589451940373444177").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("3442170070232630048373467858370187550597693105970967239471088845868856625578").unwrap(),
					U256::from_dec_str("2565836555349760764341846699730262037514609346007897474387212424068759288925").unwrap(),
				],
				vec![
					U256::from_dec_str("9177399628115629455311170363743761185656955089341861651556728182077007742962").unwrap(),
					U256::from_dec_str("8978098454385577234885149545555358562524532626359474543660036795706680306193").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("3475249974036476162964977391934788659886943458806724530569597219135120562284").unwrap(),
					U256::from_dec_str("5909974155820403611803060790871333693814260546794228782122506894217269077189").unwrap(),
				],
				vec![
					U256::from_dec_str("8948147707464445512770554781127804344587029394518818985316769045847167306053").unwrap(),
					U256::from_dec_str("6244220639119464514520028052853849971703107458448799275761796309223160549660").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("2066389909367768648597251810241027238345950972070643168750070171403496171911").unwrap(),
					U256::from_dec_str("1898304115952366298651129001635333706379006960765262098012458123450995159651").unwrap(),
				],
				vec![
					U256::from_dec_str("1856555603107176523301771843124578764710746111740718726027487402075434689409").unwrap(),
					U256::from_dec_str("7410693559992655229357237250636084977735760442337432185456859934632425894132").unwrap(),
				],
				vec![
					U256::from_dec_str("17947286896053445598360953488679420610443470034968632274205506527392450039725").unwrap(),
					U256::from_dec_str("3895334080718582330725304074745412588636689677985273210895763904111201321640").unwrap(),
				],
				vec![
					U256::from_dec_str("21416214099488461436673890273348837879747513591241316002695561057333083776292").unwrap(),
					U256::from_dec_str("3391691542348081290104434304460012299076546080317713666198018566085844483904").unwrap(),
				],
				vec![
					U256::from_dec_str("5831865841542264981997258578340410586666887289198813310234035199414406992287").unwrap(),
					U256::from_dec_str("541152292220907791488763854655135017975093511835597661057761318298652196836").unwrap(),
				],
				vec![
					U256::from_dec_str("5056168195537204983120415283750457101925458655675783189653370539259027807168").unwrap(),
					U256::from_dec_str("11956502234641567117145556474094025233261632560009796443598718512204491926824").unwrap(),
				],
				vec![
					U256::from_dec_str("6001795269485013587505281910832139630835802223081148889043717533427764920854").unwrap(),
					U256::from_dec_str("4228015156236533342169460797257057970676666487255154758089286838763712626409").unwrap(),
				],
				vec![
					U256::from_dec_str("5462314927338346325491378900451830239683587227473550602000626797213724740299").unwrap(),
					U256::from_dec_str("1884219370385302317855636369662856824751120927325767389612486204220720900912").unwrap(),
				],
				vec![
					U256::from_dec_str("13243039687919994991813543240226795294935323461312147727968488200651708291457").unwrap(),
					U256::from_dec_str("827035014682483050709645177362885116745935024256222539550691237849700735839").unwrap(),
				],
				vec![
					U256::from_dec_str("15681975845101161355157873836131193921807006515040811534580021927544498786807").unwrap(),
					U256::from_dec_str("18756207560876955742174557757015375223394082477254380529240274611994283460010").unwrap(),
				],
				vec![
					U256::from_dec_str("1882705712389915779003978943973188953955260555599932161800577389240520357270").unwrap(),
					U256::from_dec_str("8629850575835595094144763041260818362612673534288612613928346317196841467522").unwrap(),
				],
				vec![
					U256::from_dec_str("8745855082804994819863529631763084442816205249584071058941190555898569479735").unwrap(),
					U256::from_dec_str("21692253036198249194141883338115917121473268018207126949013452222295505497687").unwrap(),
				],
				vec![
					U256::from_dec_str("12613094626743071120691866963796778649837681616812682051563400684414421533813").unwrap(),
					U256::from_dec_str("9365490776656343962433114014307365201522923168991847888677225365929586030792").unwrap(),
				],
				vec![
					U256::from_dec_str("6793880159203088817715392452604750348453297413441771558492674201855922113128").unwrap(),
					U256::from_dec_str("14346139603312465753236789574853960983726518372342712328180962073472090606196").unwrap(),
				],
				vec![
					U256::from_dec_str("12253978157854975741926086916623300218115958594798658756062517256213069901319").unwrap(),
					U256::from_dec_str("15755875237340037763639033271530197989745016797836395000137836295436129566278").unwrap(),
				],
				vec![
					U256::from_dec_str("17526046055364152839416951169028421866813866182183381491456998835227689276379").unwrap(),
					U256::from_dec_str("8841542110539524297880983325365406606216349430596688342346193657692690166816").unwrap(),
				],
				vec![
					U256::from_dec_str("18615347970012612210861525771234905658961350491594352447044050613860219550724").unwrap(),
					U256::from_dec_str("3923619697613397146786407151934132719962859996691619643410472870976985170973").unwrap(),
				],
				vec![
					U256::from_dec_str("20291060583560144641358065825340025080860431713628862915256408411377719001439").unwrap(),
					U256::from_dec_str("17123445630419458544438131056773390642498133859695306119353438106606840478651").unwrap(),
				],
				vec![
					U256::from_dec_str("4202847509845861205906637237139552898497275001434929028982440441966918234993").unwrap(),
					U256::from_dec_str("10019675637336603739195974439962186867185262170660507535822968246476463670570").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("28").unwrap(),
			U256::from_dec_str("36").unwrap(),
			U256::from_dec_str("45").unwrap(),
			U256::from_dec_str("55").unwrap(),
			U256::from_dec_str("66").unwrap(),
			U256::from_dec_str("78").unwrap(),
			U256::from_dec_str("91").unwrap(),
			U256::from_dec_str("105").unwrap(),
			U256::from_dec_str("120").unwrap(),
			U256::from_dec_str("136").unwrap(),
			U256::from_dec_str("153").unwrap(),
			U256::from_dec_str("171").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_18 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_18 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_18 Benchmarking failed",
				);
				panic!("sumout_18 Benchmarking failed");
			}
		}
	}

	sumout_20 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("5656680928965688826993118194012753618765968357565979972595276559147873694083").unwrap(),
				U256::from_dec_str("6967549426192095910922952274984550698556833242157857540292539138373310729143").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("19268885367267656136131197855524200613191251527598109247559449729103347917426").unwrap(),
					U256::from_dec_str("13276555837892122843601384924562081175658016150684783638795380360267979518059").unwrap(),
				],
				vec![
					U256::from_dec_str("6192395283135706284507684938197927443456404304371530286126690358787918110065").unwrap(),
					U256::from_dec_str("8819009190264908298811100900277161411847764736308179893794014638324292594780").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("12737674974511230996771863495830727745486180187120990738854988060882540937222").unwrap(),
				U256::from_dec_str("17139125343511579901747204746002442189775589547519308944117697554234845166515").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("244263057245854881181695352829624456418211798865192445082991456290482557985").unwrap(),
				U256::from_dec_str("11379591029981368411890266449113199799040076029174714233656465956470810325768").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("21682669782412056972659921333433470180823921348801871274366645367524620903864").unwrap(),
					U256::from_dec_str("2455009268298884604476741290429694617356204701265455596321899599145219280549").unwrap(),
				],
				vec![
					U256::from_dec_str("21125883925383156333929232848711525380172926056014142857886611755830225500203").unwrap(),
					U256::from_dec_str("15135796545808937165305203902401320696776563359425008931719414027175268786847").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("15450920303903437518361455913102265455988608824135846597836221037214352737619").unwrap(),
					U256::from_dec_str("11439120571496221085972693042194450422532276411899764622108255930379779573186").unwrap(),
				],
				vec![
					U256::from_dec_str("5504373244182826481954471057557555369791621799450298835807839203848086857375").unwrap(),
					U256::from_dec_str("5801334494957666066246189629199512612886312175942824799309062312586542544118").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("1141374384314887705660896617887241019152129114063489281405071135888366753641").unwrap(),
					U256::from_dec_str("13076370956699658631269582407826656960323339094757103831961309662392298659347").unwrap(),
				],
				vec![
					U256::from_dec_str("4426949087394010699249325505509668334533507346720007620471403313387916351148").unwrap(),
					U256::from_dec_str("4708947509252113613324407090303358949370926839796559574786305350757928334665").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("7357683365595790287993998127975259977742608220801091434413870585009415257676").unwrap(),
					U256::from_dec_str("1506871282350279588306866242730945049104836644264089499533750872901945500369").unwrap(),
				],
				vec![
					U256::from_dec_str("17603107985343948702246104170283386613575028902039517361540799399845966575171").unwrap(),
					U256::from_dec_str("18820643961153442592821124056307287292205523842039764422947098696032617853855").unwrap(),
				],
				vec![
					U256::from_dec_str("17650798421825975173386758567164281203076719989952311509598797371365178680199").unwrap(),
					U256::from_dec_str("17890175687403291785665866239824911374421216792676533545119802637975578402883").unwrap(),
				],
				vec![
					U256::from_dec_str("16455874793994696207521701175245486604558008503740725889172000827379626158043").unwrap(),
					U256::from_dec_str("14031822690471685955088571269493319864920100659777524458853309643802282343571").unwrap(),
				],
				vec![
					U256::from_dec_str("20217336577454114538708064936313780793013155591226224224827483353091280187207").unwrap(),
					U256::from_dec_str("7274595912406355919980207840714132002561348049707935735401310529040588077044").unwrap(),
				],
				vec![
					U256::from_dec_str("21481014700554264388848521790017731722166888066694391737960816015477266386487").unwrap(),
					U256::from_dec_str("17455044584700880292255973227977239165349932901603612566533923980353194321813").unwrap(),
				],
				vec![
					U256::from_dec_str("21397911766364883257937607904549984527882328767474455543633946325088982450338").unwrap(),
					U256::from_dec_str("8473738210819751426980400124613437104723530057983009606149842528359834537276").unwrap(),
				],
				vec![
					U256::from_dec_str("17581651979137517837372735764325843253645494664157105398993836978138260652408").unwrap(),
					U256::from_dec_str("17769300675471550881080026982937776951432530235743530865543166960549108293804").unwrap(),
				],
				vec![
					U256::from_dec_str("3642823531932009942227226738288531842509469524513016172644199939718550077569").unwrap(),
					U256::from_dec_str("17432723582900147732334867408509412616629549151824191911587120144347800943104").unwrap(),
				],
				vec![
					U256::from_dec_str("12421919210235701164201409268688646230307360862186347052760389576025389595076").unwrap(),
					U256::from_dec_str("18330339236406203807629192609438039320235594597301963978705179812699763588479").unwrap(),
				],
				vec![
					U256::from_dec_str("19947323889374706457771950555677486901830894290183470381305433037498145547023").unwrap(),
					U256::from_dec_str("5203650088207812959138372874327437440900018099517864797228477559056193893752").unwrap(),
				],
				vec![
					U256::from_dec_str("4255951379517667170373780028202354788803406813972719150978785542172116669531").unwrap(),
					U256::from_dec_str("14451136209830519398067981987488499900976834426547453885610521940106213984929").unwrap(),
				],
				vec![
					U256::from_dec_str("3925629517135871256290787439634621941614062463495442837265339103765138337761").unwrap(),
					U256::from_dec_str("2497491815757381945749671166420308545458065641820045413388573704927695241813").unwrap(),
				],
				vec![
					U256::from_dec_str("21466120707849646160137487346724407872042410344830040590993945475592511746214").unwrap(),
					U256::from_dec_str("3045215934664432127908841075160905018526084085596312155808606706810118424050").unwrap(),
				],
				vec![
					U256::from_dec_str("17180695787385241592651918696972083237984540030182966929320858606796263773636").unwrap(),
					U256::from_dec_str("2049214809205333765116872668089154986377762659390367408031763978502033709374").unwrap(),
				],
				vec![
					U256::from_dec_str("4774299874040054677277833517513219583032845720635034514759165863256091223071").unwrap(),
					U256::from_dec_str("2817429278103215029312280393926618707332349894417886654266983777338602669217").unwrap(),
				],
				vec![
					U256::from_dec_str("11849511390994459187777805386685940997731436566748603872125066265609444105290").unwrap(),
					U256::from_dec_str("15749060430488318934192185845451610512322384322770593887198388744891795166876").unwrap(),
				],
				vec![
					U256::from_dec_str("6816330773235217141791322362184115309193655994447714036407608955587670289691").unwrap(),
					U256::from_dec_str("6202757244116766574486999165436015492276224399898135936516804984343474464324").unwrap(),
				],
				vec![
					U256::from_dec_str("4472916876847760069533088181774934171702146143581240908666459515397433940327").unwrap(),
					U256::from_dec_str("16847088658324790629288327397928655196248049022361853173150753668961227880157").unwrap(),
				],
				vec![
					U256::from_dec_str("4704407312269183027827709279804690513392733846890090565911802030907318348957").unwrap(),
					U256::from_dec_str("12023785273031858079896145099534749991123742716328675854453794204222375745635").unwrap(),
				],
				vec![
					U256::from_dec_str("14489517201954926339160541383066537697773411650355771620316229200747554266343").unwrap(),
					U256::from_dec_str("1201374487316207220705666090779729243474012798987516363690861140880166104963").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("28").unwrap(),
			U256::from_dec_str("36").unwrap(),
			U256::from_dec_str("45").unwrap(),
			U256::from_dec_str("55").unwrap(),
			U256::from_dec_str("66").unwrap(),
			U256::from_dec_str("78").unwrap(),
			U256::from_dec_str("91").unwrap(),
			U256::from_dec_str("105").unwrap(),
			U256::from_dec_str("120").unwrap(),
			U256::from_dec_str("136").unwrap(),
			U256::from_dec_str("153").unwrap(),
			U256::from_dec_str("171").unwrap(),
			U256::from_dec_str("190").unwrap(),
			U256::from_dec_str("210").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_20 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_20 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_20 Benchmarking failed",
				);
				panic!("sumout_20 Benchmarking failed");
			}
		}
	}

	sumout_50 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("13827879866301415538857432999218866982628860414954597546568484592640037099744").unwrap(),
				U256::from_dec_str("14551999579990087654454739665634365604997695433710312891076467259913934388803").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("15771353158313563683613152088814408294817138752077129512988787477344110757188").unwrap(),
					U256::from_dec_str("14399279855844834913268182033927156273557513850691671069599176570477483195735").unwrap(),
				],
				vec![
					U256::from_dec_str("12275212562972350055374119845928748810285249001807183087376345867226594250614").unwrap(),
					U256::from_dec_str("13756757166577921760248781523737155114632832632168220989358664582399059540764").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("7744904145895296256394657338969049284442931947677085357490462223272611262824").unwrap(),
				U256::from_dec_str("6220983930078260337427503798082895905560328943244814949511556092413235993784").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("6115335040929689045763833235655184287313807320983573556493260194789528971686").unwrap(),
				U256::from_dec_str("6166532138119800856613629046632439223273859498423658929523110122960131290393").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("16072185984981940759925604108298758265427751820417866991432783389012340676920").unwrap(),
					U256::from_dec_str("12038526475382871968757930147404918510943224535062619233972655078048128772207").unwrap(),
				],
				vec![
					U256::from_dec_str("3357579678449381536683529270370009397075211562688187149111391775074561165741").unwrap(),
					U256::from_dec_str("18242626048326784141288643701409962774080323384992795292954067136702849374522").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("8666829685645767563314329202552522059811973795622694031857713490395410267737").unwrap(),
					U256::from_dec_str("11859365458670977483026426220940789952575696888959917148983915658234029017108").unwrap(),
				],
				vec![
					U256::from_dec_str("14130230782411277014762522839402392302490685104885373749038912991243039063851").unwrap(),
					U256::from_dec_str("8372970790028263146112897121060860570673044309366330703937159104057494533039").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("17810070930649725073550928999479437396879560471136036549038763141608403698246").unwrap(),
					U256::from_dec_str("5892003699669605510258891634989426469331530174006810427612794736057219035068").unwrap(),
				],
				vec![
					U256::from_dec_str("17627382974715312497434404825093389646496261744940225730753163482625180535264").unwrap(),
					U256::from_dec_str("8810117541294897071211549144170262855239024685766481396194935888376586571442").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("657738098560847380495053334619058455958793334766494917034230275292791995444").unwrap(),
					U256::from_dec_str("532105491296268554235023930179690358089560316352213261084871842896383948711").unwrap(),
				],
				vec![
					U256::from_dec_str("1097662479221345957152012749497455033885787513971827778405858673808706126572").unwrap(),
					U256::from_dec_str("2447207493658220005172443593380979137893421348377510383419810991856073882654").unwrap(),
				],
				vec![
					U256::from_dec_str("14987702786869929603736432978965272287140970095658077826321786680678257701539").unwrap(),
					U256::from_dec_str("6623830625206727823787295668755924167882248998617207072323403593721086329595").unwrap(),
				],
				vec![
					U256::from_dec_str("16083000893807119940364492004763007744340588623320271607738723759855697960706").unwrap(),
					U256::from_dec_str("14441879876890467114234414051431347036645300127848755680103166207990794576758").unwrap(),
				],
				vec![
					U256::from_dec_str("15731029203716777672456183943350227025919210853689017608162251677628010445863").unwrap(),
					U256::from_dec_str("3243323103776869275672187227239319642124956814578013158853719229338650062946").unwrap(),
				],
				vec![
					U256::from_dec_str("11507951232495383993382480534635046047566127951931310314719985521726794994855").unwrap(),
					U256::from_dec_str("16521537108350445831368232156819046644534933711158400995311389097851953323826").unwrap(),
				],
				vec![
					U256::from_dec_str("14900110644751936433863090329189868498744457501863570858210025530486652384145").unwrap(),
					U256::from_dec_str("14359597649211951395870532227205969995295738578955836733248597883152057352152").unwrap(),
				],
				vec![
					U256::from_dec_str("16088110181600531322538582358406898104352202099807306598784290518774717043236").unwrap(),
					U256::from_dec_str("1939008393245671182845407198815819570976145968765277955677921816083337658264").unwrap(),
				],
				vec![
					U256::from_dec_str("1716154303829123720378392190766578286599416024855264465881476079904832192454").unwrap(),
					U256::from_dec_str("15136578653408806943816555010155427299402698229965663640184024060430500322706").unwrap(),
				],
				vec![
					U256::from_dec_str("21607870289382776942440929696746256287606813134801627227394120993907081115482").unwrap(),
					U256::from_dec_str("15216673476758896215159140003517149726322470027268268399303559990796350851668").unwrap(),
				],
				vec![
					U256::from_dec_str("3470717514306671402640704490357575134271293191039101496626686232282200916588").unwrap(),
					U256::from_dec_str("1644143157682464475771925146618463012305479502919125062639366962721572466038").unwrap(),
				],
				vec![
					U256::from_dec_str("14141951040433426732708898160256492540100390122590931176192871614825767567823").unwrap(),
					U256::from_dec_str("14970870554510027003883479850485276956804732428074135837976610927084756596698").unwrap(),
				],
				vec![
					U256::from_dec_str("6757052599412251823187168407425869720480978223208220243035283923157486962640").unwrap(),
					U256::from_dec_str("19998998568187199475005177265892129077143217765879194219305910411258469180208").unwrap(),
				],
				vec![
					U256::from_dec_str("20881696828098941844228741422556682968632634770060670451389297054104176440621").unwrap(),
					U256::from_dec_str("13608889661472160976940462761860587603342008277901142604062787628433773176010").unwrap(),
				],
				vec![
					U256::from_dec_str("7213318721268961019485708544153236686954566374506720345579959324834177101641").unwrap(),
					U256::from_dec_str("19592958466387187227162933705639453686422518128902348109190215207992371486918").unwrap(),
				],
				vec![
					U256::from_dec_str("11492956236335799028306792618137865677945795549817827579564445070141572466907").unwrap(),
					U256::from_dec_str("16711820022758729063513408191192188870792118206420418650892132773179116683557").unwrap(),
				],
				vec![
					U256::from_dec_str("20945069163147606665108872405227233214878505928402156102335106047734925593215").unwrap(),
					U256::from_dec_str("1423987675745181080834644795321752693594601107027563001974881039188492021171").unwrap(),
				],
				vec![
					U256::from_dec_str("15891267199767390072540329971020389290472310562561860106988736756158294968407").unwrap(),
					U256::from_dec_str("2267503153642457690523865861608670312822668575934691972853239401502939363062").unwrap(),
				],
				vec![
					U256::from_dec_str("7474594013463066171479789286133270044299000414563424286960176499673549013911").unwrap(),
					U256::from_dec_str("8122674047183480331694645349931807273093092753341828673326730648947527889665").unwrap(),
				],
				vec![
					U256::from_dec_str("10257111743245921777033251489771345706287334159675215734672219144502175080691").unwrap(),
					U256::from_dec_str("9949970303925511520216099354542476962005374285896059168560924362910911093080").unwrap(),
				],
				vec![
					U256::from_dec_str("19537041589568134851029831433550253908067307517024871465298468340397396673864").unwrap(),
					U256::from_dec_str("16858223705695697450984778594276887015292986391267341267776294917186229957278").unwrap(),
				],
				vec![
					U256::from_dec_str("15246698552087261534042715485066570376107106051211884589703176775025917332690").unwrap(),
					U256::from_dec_str("10750598760851770035286468210307770266777083135480137548147767265257368199367").unwrap(),
				],
				vec![
					U256::from_dec_str("20630242320426959800851643081479738249548329066422905492655846397008060618115").unwrap(),
					U256::from_dec_str("16326371293493241977282281258423453788627821631429026408337594541860635510205").unwrap(),
				],
				vec![
					U256::from_dec_str("9259379886124890427913666590932019503885682624879306600117835935139457180688").unwrap(),
					U256::from_dec_str("11581379817981971916924055685584021099145296077431456350129178254038600126989").unwrap(),
				],
				vec![
					U256::from_dec_str("6611770750163234216247925941306289769928406522729441385143389903289545931187").unwrap(),
					U256::from_dec_str("16388238514629781365495798832142666963626297616941516091917404053288951061020").unwrap(),
				],
				vec![
					U256::from_dec_str("16643609599328489508845877463250216488035062509642391199018340115352661317445").unwrap(),
					U256::from_dec_str("19287928770096457485364071157830517856406460763445755502553439072821026326160").unwrap(),
				],
				vec![
					U256::from_dec_str("16668268135822983565074541614143405097045096145537034830204790679728078054636").unwrap(),
					U256::from_dec_str("1066083751611943963019573342152116287954637626803138791681744825116321882626").unwrap(),
				],
				vec![
					U256::from_dec_str("17564263958855568483001610697806473789956400903711217865830758776454673450601").unwrap(),
					U256::from_dec_str("4296134423505578747039658012410131620047948001497680469667933158152933142610").unwrap(),
				],
				vec![
					U256::from_dec_str("504256330700931929914991961911525033003353677755203482879812398706411650385").unwrap(),
					U256::from_dec_str("20984100739676535922476274553914885299249152392042051753547982635038546311916").unwrap(),
				],
				vec![
					U256::from_dec_str("3869421570438658969494179519379461979744673196979406788310235907475541151866").unwrap(),
					U256::from_dec_str("10384783961201503389960224508131172245984891572889398757617253808540561858064").unwrap(),
				],
				vec![
					U256::from_dec_str("7122215811122822268507423970767532589760454224609641145825381146476966978386").unwrap(),
					U256::from_dec_str("18629120702848018429623470369106570770627862115848413285088767540695603529504").unwrap(),
				],
				vec![
					U256::from_dec_str("17761362406558450236735871511552593182707619093161051811749087853668432576447").unwrap(),
					U256::from_dec_str("19034593786971827440873811266124476710852272184042594489649531393766747348571").unwrap(),
				],
				vec![
					U256::from_dec_str("15804166932378673108559283693463783257192568677741795632273990124666564682412").unwrap(),
					U256::from_dec_str("11551825304007398963235970620449268090281036794735216055254821775490144519805").unwrap(),
				],
				vec![
					U256::from_dec_str("21646450176753271316838399881606539827387849682556972105696360925302077438232").unwrap(),
					U256::from_dec_str("2511076513821207007348582223879937756955220822617888786870823449323773645256").unwrap(),
				],
				vec![
					U256::from_dec_str("16904410309855743683736810330775505669696719943988498514719208857633212308605").unwrap(),
					U256::from_dec_str("21324333522424177499994582360198666554142246038524559672999661787897876060143").unwrap(),
				],
				vec![
					U256::from_dec_str("16945490191295215569999901067842377598942481617410852961564362274204107146706").unwrap(),
					U256::from_dec_str("18314156748004089222492732633319765815006654008818903454759444358188055539728").unwrap(),
				],
				vec![
					U256::from_dec_str("16917767336476454814446678739488168052806439126778827398781629030350151857387").unwrap(),
					U256::from_dec_str("11342172614570752746108213362461158451964820583854525536745594715060784857536").unwrap(),
				],
				vec![
					U256::from_dec_str("19655214901040692360358163384134838077837280723734255140594660062110580656545").unwrap(),
					U256::from_dec_str("3362419428602909135476998407729641861009514940918624516093829774638749378477").unwrap(),
				],
				vec![
					U256::from_dec_str("10068364657928239999688712782881075590100091150035478948591571494298540064040").unwrap(),
					U256::from_dec_str("3925841392732674526382167900635378545357933312779748980213399765561568501954").unwrap(),
				],
				vec![
					U256::from_dec_str("7859837372449593447197076679734892380978595170122834233920177020113058884162").unwrap(),
					U256::from_dec_str("15267327488914432697481346031072906103448848928991578280835939381635251283161").unwrap(),
				],
				vec![
					U256::from_dec_str("17820543025474357991914589452937303503868675134182823718184408612418457106496").unwrap(),
					U256::from_dec_str("4080173097768027859462877326409054600046050536583611074603903692960179295306").unwrap(),
				],
				vec![
					U256::from_dec_str("7222395274549846905402287163598280036522206923543195587339746656511726156757").unwrap(),
					U256::from_dec_str("17997737062514751282939534276628529489984717427968026247212791392586232604551").unwrap(),
				],
				vec![
					U256::from_dec_str("1711582578037897629120492811026654304279972642865731976334484807734905010624").unwrap(),
					U256::from_dec_str("11417750330605984209119442624747765360816092826690016986275336228747618648201").unwrap(),
				],
				vec![
					U256::from_dec_str("5091961738279253769908069837603749933382976381492972380907447338157526366714").unwrap(),
					U256::from_dec_str("2098634419436065415091734819109653428284310850731241021115434822766452990108").unwrap(),
				],
				vec![
					U256::from_dec_str("12771630983152860380958654612308310234837340761565253596054160793193407831416").unwrap(),
					U256::from_dec_str("9149694848418949644717518759058810074518704169947141770421799720539197666625").unwrap(),
				],
				vec![
					U256::from_dec_str("9918837719060277739930324221392324331275666891900263953129450726331466863134").unwrap(),
					U256::from_dec_str("13829770172279463687649225699345042056345548299961998544225247740202252340439").unwrap(),
				],
				vec![
					U256::from_dec_str("21556772538865001220884467378626718237742195027485835738020776901730681324637").unwrap(),
					U256::from_dec_str("19778672038763116743630250909961650038370256095030032529759391735347222925746").unwrap(),
				],
				vec![
					U256::from_dec_str("14856861307423975278581613005761934562358117742422665247736527476263653360349").unwrap(),
					U256::from_dec_str("1867918246775206089638452481671058455990990896563653326270871198890663017878").unwrap(),
				],
				vec![
					U256::from_dec_str("7358292501557376813334144315791019321478041455375833895705472993641179430655").unwrap(),
					U256::from_dec_str("19639382277696759182661917452231356435729059958722755277286321238745721897653").unwrap(),
				],
				vec![
					U256::from_dec_str("5776957264346829457305929424782026635416082038398352721156570341141914144551").unwrap(),
					U256::from_dec_str("15767534494868786069535421819233640712284308687242022324923974165520151035749").unwrap(),
				],
				vec![
					U256::from_dec_str("15314810752842818694345949990176494977521994584277889144518086930381852217499").unwrap(),
					U256::from_dec_str("7856791107683021983787028049162290107091787348782025165222243877660987556554").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("6").unwrap(),
			U256::from_dec_str("10").unwrap(),
			U256::from_dec_str("15").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("28").unwrap(),
			U256::from_dec_str("36").unwrap(),
			U256::from_dec_str("45").unwrap(),
			U256::from_dec_str("55").unwrap(),
			U256::from_dec_str("66").unwrap(),
			U256::from_dec_str("78").unwrap(),
			U256::from_dec_str("91").unwrap(),
			U256::from_dec_str("105").unwrap(),
			U256::from_dec_str("120").unwrap(),
			U256::from_dec_str("136").unwrap(),
			U256::from_dec_str("153").unwrap(),
			U256::from_dec_str("171").unwrap(),
			U256::from_dec_str("190").unwrap(),
			U256::from_dec_str("210").unwrap(),
			U256::from_dec_str("231").unwrap(),
			U256::from_dec_str("253").unwrap(),
			U256::from_dec_str("276").unwrap(),
			U256::from_dec_str("300").unwrap(),
			U256::from_dec_str("325").unwrap(),
			U256::from_dec_str("351").unwrap(),
			U256::from_dec_str("378").unwrap(),
			U256::from_dec_str("406").unwrap(),
			U256::from_dec_str("435").unwrap(),
			U256::from_dec_str("465").unwrap(),
			U256::from_dec_str("496").unwrap(),
			U256::from_dec_str("528").unwrap(),
			U256::from_dec_str("561").unwrap(),
			U256::from_dec_str("595").unwrap(),
			U256::from_dec_str("630").unwrap(),
			U256::from_dec_str("666").unwrap(),
			U256::from_dec_str("703").unwrap(),
			U256::from_dec_str("741").unwrap(),
			U256::from_dec_str("780").unwrap(),
			U256::from_dec_str("820").unwrap(),
			U256::from_dec_str("861").unwrap(),
			U256::from_dec_str("903").unwrap(),
			U256::from_dec_str("946").unwrap(),
			U256::from_dec_str("990").unwrap(),
			U256::from_dec_str("1035").unwrap(),
			U256::from_dec_str("1081").unwrap(),
			U256::from_dec_str("1128").unwrap(),
			U256::from_dec_str("1176").unwrap(),
			U256::from_dec_str("1225").unwrap(),
			U256::from_dec_str("1275").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_50 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"sumout_50 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sumout_50 Benchmarking failed",
				);
				panic!("sumout_50 Benchmarking failed");
			}
		}
	}

	fib_3 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("3132971325870222259639561706189758098287244657388649414392529352718303714530").unwrap(),
				U256::from_dec_str("6507826058647329464322284230325703133591122928282742834699944707019156258809").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("17772115053309497146137293588980018321863662294867005210136855479605214186180").unwrap(),
					U256::from_dec_str("10038179076166881173143876188899696762808386040833545597371587600563515084302").unwrap(),
				],
				vec![
					U256::from_dec_str("84866774058471735936350793806054983511272962266883047883471400112941191037").unwrap(),
					U256::from_dec_str("5568809559395193198199152670834662954559187185156161953287752106001302001344").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("8880668092662582662151749297267512620096355234087050144351597799319681896574").unwrap(),
				U256::from_dec_str("11528980930108519143107616915390840029128779349741456778024327951728523777236").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("11973149780461845670977457832614205154706068463789633369928681722573108265443").unwrap(),
				U256::from_dec_str("560154769206684285089883239395400651095151098880172398808069275901669863769").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("13756428714932434595021589136752822391122965542123120193354239911120469699709").unwrap(),
					U256::from_dec_str("120112527160047884703620942006841978570569214359122057960251258287184219697").unwrap(),
				],
				vec![
					U256::from_dec_str("13463898713692437806260449684498030384996189360575139011629743781467141790304").unwrap(),
					U256::from_dec_str("1194002625152303589071335639210384071235225241955701801770234611166742017055").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("11139054187995416626739127332337577172136803991433018900962790056334353344291").unwrap(),
					U256::from_dec_str("6070725161016897987520708628733735347166393581684474326320014699046060380622").unwrap(),
				],
				vec![
					U256::from_dec_str("19209700656348280109959147287216993350201779451119908883346282260552870356387").unwrap(),
					U256::from_dec_str("17266437688499472585954564896738180051466183385397662700252060765908415760821").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("8518978947911781334617073974199814521245393345576825252918283414965139226281").unwrap(),
					U256::from_dec_str("1919930890570357741377145521418378700589359366071674476012553393210299710266").unwrap(),
				],
				vec![
					U256::from_dec_str("20518172778897855607538153208553426015126061748797486778622889113076225553601").unwrap(),
					U256::from_dec_str("4341433561739154626175616238599696777704881435167300120740573786688737320206").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("6257454378471671954508216959821760138899506248862124107543881832751496243381").unwrap(),
					U256::from_dec_str("849484394494738364726633009377512849646844450581834914681550559373523098955").unwrap(),
				],
				vec![
					U256::from_dec_str("13065948243673575219205285761966669534222259117764240672025250386712503766186").unwrap(),
					U256::from_dec_str("14256307983349064955937850546866094921248521794981704681964738514708851084548").unwrap(),
				],
				vec![
					U256::from_dec_str("11615738609008198513128136474869049796581791264774401388480327383708701220818").unwrap(),
					U256::from_dec_str("17975851278557099428627721105403634331615370696748419056473417067984791293951").unwrap(),
				],
				vec![
					U256::from_dec_str("17708563515128206907836464584450502652129276662274674459408979775646454697813").unwrap(),
					U256::from_dec_str("19932771729628827926695230158177139025768662902055116478576202138537488865502").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("2").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"fib_3 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"fib_3 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"fib_3 Benchmarking failed",
				);
				panic!("fib_3 Benchmarking failed");
			}
		}
	}

	fib_5 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("7107575577163000349795659359956168176171101387530780151824981638484262486978").unwrap(),
				U256::from_dec_str("897133249212133391804343723156610138182460716033738784908100029698973225852").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("170389711510642910317680110863332628397432593441060800544776128980970914252").unwrap(),
					U256::from_dec_str("6920395653366563258534283590246960198394308227032492249037643353403270317045").unwrap(),
				],
				vec![
					U256::from_dec_str("8480799764075268077172001588258796963309752386101311077889514097122873239882").unwrap(),
					U256::from_dec_str("12451695893898630503865218111936486188627157726164511249977202673656946431620").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("21597992290581954545776627017098028702839071011677341576724377506987149993751").unwrap(),
				U256::from_dec_str("19053567793179926208306730705211711754174669587802570078364147255403467780151").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("2357887722618418674206341083287923328601312919804823733855833233026791743663").unwrap(),
				U256::from_dec_str("975374039805822885844240552519100209502334213678144709852000084502893973008").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("16280105556148393779575170892506606219120071731060039616507494801779121328832").unwrap(),
					U256::from_dec_str("20950039635055833909912134692658157347220364503511282528956587902894228703712").unwrap(),
				],
				vec![
					U256::from_dec_str("7993964715201116449357589101743367175186265961068472625342038948672634732296").unwrap(),
					U256::from_dec_str("11099581941940578041819214468757040086707821891633617156886035885244647996327").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("12636329372753298988357489028292457750683260728540316442371004560814413226626").unwrap(),
					U256::from_dec_str("4045455970628132355106370857917185793087210373549168231277818312754971612807").unwrap(),
				],
				vec![
					U256::from_dec_str("1300344419079427628501727843429767670124424070337281261058615696156550837549").unwrap(),
					U256::from_dec_str("2174525330618542624943912040794362533422023824780024477106228987295196169137").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("21390433795647917247027254319423864113578346839713218763993047007421497228495").unwrap(),
					U256::from_dec_str("21661918443945562352764731942118046974455095191616778616511655172171711481914").unwrap(),
				],
				vec![
					U256::from_dec_str("6520633594028678038498884349163102469806602416433112898771095987637595066617").unwrap(),
					U256::from_dec_str("19663631449513124110735458456019142408282532497123847160755982598286201463717").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("18657381897074200796904415404231863341895995632749476893629144717257679103963").unwrap(),
					U256::from_dec_str("8345650724109397616811518344854358214988563616857860428608140138066962138957").unwrap(),
				],
				vec![
					U256::from_dec_str("6080713600081003917101685986497728921078282571010625261265667442187252662937").unwrap(),
					U256::from_dec_str("513944790500250868640338307249655484768530127653315385350721456220511760102").unwrap(),
				],
				vec![
					U256::from_dec_str("16799415331247767599223181727867706342904206821513386749641266330877174898760").unwrap(),
					U256::from_dec_str("19420916090388399599536229341092487229974297531937960707965143244382507176964").unwrap(),
				],
				vec![
					U256::from_dec_str("13668334697827007947302996095199479530567641377939668790543641044451953957139").unwrap(),
					U256::from_dec_str("12476894726233017534672279206882408584567103640359862711942440003398820167147").unwrap(),
				],
				vec![
					U256::from_dec_str("13809977379539470670333778802717132743390915159675198170487848397796913472215").unwrap(),
					U256::from_dec_str("5304572538397463768441415723346479145852763698142825603512106636952539776032").unwrap(),
				],
				vec![
					U256::from_dec_str("701521285562685954268529162647895469084095604096437475664109908959446258033").unwrap(),
					U256::from_dec_str("15172311417293466130771317503680468656450805959709148012902980907068100374524").unwrap(),
				]
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("2").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("5").unwrap()
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"fib_5 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"fib_5 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"fib_5 Benchmarking failed",
				);
				panic!("fib_5 Benchmarking failed");
			}
		}
	}

	fib_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("15598002460690588437896426762633589755681380926526826007650911266155302746971").unwrap(),
				U256::from_dec_str("14048781467582620915514707321102379170342351199203838720084613707384189246618").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5701442717558059864094936857346939502743862376737227680931736705155409754850").unwrap(),
					U256::from_dec_str("6705418082619681590489582801426443732890080590377224079621881818162843204595").unwrap(),
				],
				vec![
					U256::from_dec_str("13279521252289501630047789160726986062397338623708136865134983184465749454165").unwrap(),
					U256::from_dec_str("21033452491153645343877459007984264553702116582842254942279478167695403794865").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("4781563429922683357122820523637836932301917765582180198660942288815260302421").unwrap(),
				U256::from_dec_str("14814729180207128675172446059166803635423524097383393305814823960659595806329").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("21128226372968921480959222733647870812725374627959087109938949205782449934509").unwrap(),
				U256::from_dec_str("7018642648086750325301120070974798999557335793924792347011375693170409195098").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("10332150704003579947885996835615852693671986689983776582911110245805513692668").unwrap(),
					U256::from_dec_str("1124448241753524378997499989602955777745777344553269747221648865489793488975").unwrap(),
				],
				vec![
					U256::from_dec_str("18359851081925516932372626627155466137254869746456141360772489145074657585390").unwrap(),
					U256::from_dec_str("2503686322149969597053196188893305621132468879410881045066232089347592198484").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("4816278985941566400895633011850808885261900890766698900964554024095285628690").unwrap(),
					U256::from_dec_str("4481223810973703865132739390383257176145788880766036414318913695394582687416").unwrap(),
				],
				vec![
					U256::from_dec_str("15603469913936514792797227253072839537192706677995431175913126336649422890069").unwrap(),
					U256::from_dec_str("21864594615695298757460783973470516017964190218336035301988952994099300002552").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("21231927944393920243160099971507175021287196370917268996863763412686284403869").unwrap(),
					U256::from_dec_str("11235057574641688951864488055161587538057015254360438255114278019066405496417").unwrap(),
				],
				vec![
					U256::from_dec_str("13775234459674435911239591565261874307122702001326411862915067861775676454154").unwrap(),
					U256::from_dec_str("16068368840115781322039412863465620735514071242596879769482158447800688468756").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("8946500491146824741411381190131650044873203574966470669798623284509235212225").unwrap(),
					U256::from_dec_str("9207778656916363948673619648074334783746856414577233256897457918279555367126").unwrap(),
				],
				vec![
					U256::from_dec_str("18229827395116865835955974167555190610618187087749776491709874843200696280698").unwrap(),
					U256::from_dec_str("5354841661231629559124068306361245380218908391918092200894031389928119849378").unwrap(),
				],
				vec![
					U256::from_dec_str("8406088130432915530837275071042043342847231172695656108751797564706756513873").unwrap(),
					U256::from_dec_str("1692113603002985183084671804623810248784246597585146031631850722491435245828").unwrap(),
				],
				vec![
					U256::from_dec_str("3781409869223206029979945624869146198489017871864132897653633688094104894010").unwrap(),
					U256::from_dec_str("914649635961084769359381892125208118471978903853445450960187255648945554484").unwrap(),
				],
				vec![
					U256::from_dec_str("4681412300653217556730318589043345985554894195960903393718460114249743089179").unwrap(),
					U256::from_dec_str("21360858821457283911073676012703566640289750014744439028328185444091026027882").unwrap(),
				],
				vec![
					U256::from_dec_str("20897966082308332377807565195025690308345926928964539367186230582160325134474").unwrap(),
					U256::from_dec_str("9626498089212728094225083170299883982718296909352243901431927962362382218927").unwrap(),
				],
				vec![
					U256::from_dec_str("14359853109533518444168562509201102039176753094698437798285771319619295861708").unwrap(),
					U256::from_dec_str("90829535353937971257007177217967887697692463940020220682809349058293661638").unwrap(),
				],
				vec![
					U256::from_dec_str("6493829119246187593593139749123040079754197652894022722457052662159433878735").unwrap(),
					U256::from_dec_str("15262423895688782694421350413428209962260663475050857280745519572187099177290").unwrap(),
				],
				vec![
					U256::from_dec_str("5720350053817277761103752188290987255397596237781800199881859365664717896938").unwrap(),
					U256::from_dec_str("18530651271212069093450925444649953034166690271614200467237140911087116714410").unwrap(),
				],
				vec![
					U256::from_dec_str("16749960532530207200560191146014575700596418430016017388393265928266457503270").unwrap(),
					U256::from_dec_str("806706467165047542403196455896239214513895671141081869269826693258304989974").unwrap(),
				],
				vec![
					U256::from_dec_str("5556621093836642227227787846531768876794773395668568501898035445111568303923").unwrap(),
					U256::from_dec_str("16207931580028655113776293873436654483020022939317876762036644049054941054638").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("1").unwrap(),
			U256::from_dec_str("2").unwrap(),
			U256::from_dec_str("3").unwrap(),
			U256::from_dec_str("5").unwrap(),
			U256::from_dec_str("8").unwrap(),
			U256::from_dec_str("13").unwrap(),
			U256::from_dec_str("21").unwrap(),
			U256::from_dec_str("34").unwrap(),
			U256::from_dec_str("55").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"fib_10 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"fib_10 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"fib_10 Benchmarking failed",
				);
				panic!("fib_10 Benchmarking failed");
			}
		}
	}

	copy_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("16390962836043729391713651706041034686233675080495652797663011252649219608453").unwrap(),
				U256::from_dec_str("6865787639719296962717876412738042601880427078298243582467975656325852949218").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("17210141815814974498912537431655049857450919262672383343442093136850563838206").unwrap(),
					U256::from_dec_str("12342441380929407693451343058085382102105226929110424541171233890701211265557").unwrap(),
				],
				vec![
					U256::from_dec_str("14791256228758178288432683409274509098808992656590403960492149355352931241876").unwrap(),
					U256::from_dec_str("9757744402966524420081791838465495321308702736965720459936221854562488721275").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("12074129776039590906909589024919710171422616332187991404856704813889669348927").unwrap(),
				U256::from_dec_str("987324030858231431967082277102247672880769004615236440649645498996213854200").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("17240412344702891229776489031231066040782132824948269228246148252519138466366").unwrap(),
				U256::from_dec_str("15893975335803942380118071522945358115037231080796902411459398036409803599182").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("1271417771372629644611717230761961857698909743161160107041194135970629739863").unwrap(),
					U256::from_dec_str("4939081731814081619963862075681020612775752067767844815503719074388485924144").unwrap(),
				],
				vec![
					U256::from_dec_str("15763747362287654829011432461657723100243353639302509576933192822234504950406").unwrap(),
					U256::from_dec_str("10562112977940817988663571736399452005642324818400887824616415414761510289790").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("6840359036458076382079357592817160014882757907637013803537268679884532127606").unwrap(),
					U256::from_dec_str("1346535467813261885766726379794214479097664664392345988272417354157960728671").unwrap(),
				],
				vec![
					U256::from_dec_str("19427354612139857884342478002431397034023252622750272907992730527833498646839").unwrap(),
					U256::from_dec_str("5337050117201999843103633899784755367092656102396877506066058827688289940046").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("20823611397282852479822978978685532830296682705009136373592940707301503646033").unwrap(),
					U256::from_dec_str("7789186167969658306688153593800998736943152689699161855707311457653271858926").unwrap(),
				],
				vec![
					U256::from_dec_str("5844206256345363695439174408404096072235672265143152167172459387708567781148").unwrap(),
					U256::from_dec_str("235843202490932637756232553215192204134845956931540834734914009496237634184").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("20550659483735211121802551760554631025807244045676695123218122991238619501699").unwrap(),
					U256::from_dec_str("11945951364844791529623571555332639031054156303670814242701717081599466357872").unwrap(),
				],
				vec![
					U256::from_dec_str("654879587106775033263721472009262072007272314520227570335994171228983103986").unwrap(),
					U256::from_dec_str("14361342781229557997760264881879944101650636599963169957537169085125448452547").unwrap(),
				],
				vec![
					U256::from_dec_str("11133904625121643992823528155721740987974874506160472484890920541446857113467").unwrap(),
					U256::from_dec_str("4436717642021319260725328966952365487651517574980287441477089608456027630542").unwrap(),
				],
				vec![
					U256::from_dec_str("16685780983916865650387043033283449729899552702220387957248765192791587962503").unwrap(),
					U256::from_dec_str("1956353155823917762807403460270916420225712366921387635794647149170454029020").unwrap(),
				],
				vec![
					U256::from_dec_str("3167458143773351335886873071351828587468972050558968173198746552524979899343").unwrap(),
					U256::from_dec_str("14706542627970265967501101570095076370616715291368867838796404315730798645263").unwrap(),
				],
				vec![
					U256::from_dec_str("3169288635570110804535167586039046075463922680257787193769750793599418836937").unwrap(),
					U256::from_dec_str("6027690424292016493627852337260456340093207521914688143324731183496975625359").unwrap(),
				],
				vec![
					U256::from_dec_str("16745129804724758320492257405054914946034714396740595023799347270606765091207").unwrap(),
					U256::from_dec_str("5179934402554548729647017662939357756072702074261387023337145559991485834818").unwrap(),
				],
				vec![
					U256::from_dec_str("20612137209143666732486995940592065546560294302298353386853309983097185298163").unwrap(),
					U256::from_dec_str("5861820384642620925499126057189223472137276803184996411582299498362694892662").unwrap(),
				],
				vec![
					U256::from_dec_str("5151144014489768285719815999895590607805899943411423671520418528161058212389").unwrap(),
					U256::from_dec_str("5021097391934366383971983202419604465348564028049147673972539360521330534532").unwrap(),
				],
				vec![
					U256::from_dec_str("17919926668723224822853766036925499138771026625911650899155968406167299184157").unwrap(),
					U256::from_dec_str("16822830740823282288084447970705913414694049222039026855174213870807229940846").unwrap(),
				],
				vec![
					U256::from_dec_str("11506678974796799469204749431033361084874528449390062358406908850400100405067").unwrap(),
					U256::from_dec_str("15362354576852649604151397236428588651091047930604360653191543795436089896761").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
			U256::from_dec_str("10000").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"copy_10 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"copy_10 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"copy_10 Benchmarking failed",
				);
				panic!("copy_10 Benchmarking failed");
			}
		}
	}

	copy_20 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256, H256};

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();
		let contract_address = H160::from_low_u64_be(0x8888);

		let proof = Proof::new(
			vec![
				U256::from_dec_str("2379977889230914820672594034039121593840586279251765411407892083495964290586").unwrap(),
				U256::from_dec_str("19717766810117850874712820192154455597011669501170003294244878423890590018861").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("3973159193607304635677110039121846796616184019135954866315181300786429744636").unwrap(),
					U256::from_dec_str("19738101583692327553695201324955632910514950057767721207797779165572562320591").unwrap(),
				],
				vec![
					U256::from_dec_str("18009039990446914928726774542606020968862991368419340325967317177749500045223").unwrap(),
					U256::from_dec_str("9293138879208966557577361306623499801334435975158650451755325697234789178182").unwrap(),
				],
			],
			vec![
				U256::from_dec_str("206777167290693892804178251336642443297985114484409374966071658530105269696").unwrap(),
				U256::from_dec_str("2267483584645623390884362686034664874106799646106704430718053204392987070526").unwrap(),
			]
		);

		let vk = VerifyingKeyComponents::new(
			vec![
				U256::from_dec_str("18186832346943283968189021374987462202481085857954829692288945756447069609876").unwrap(),
				U256::from_dec_str("5402524640789301913788019447399651923814806381669718491322583849494894802268").unwrap(),
			],
			vec![
				vec![
					U256::from_dec_str("5528524380609293441422858282765117427241355019381336135853739470431762592324").unwrap(),
					U256::from_dec_str("1508226221479911609651693048768866341319973826577926330664018889585487258525").unwrap(),
				],
				vec![
					U256::from_dec_str("4028947148020159142728283348567876171081494727087452959443137954265098854797").unwrap(),
					U256::from_dec_str("4215883472206722706123802147720262143796686837980046137743755262651347678843").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("18358409527104302643151877377191921884362264066571296795400828166440394581356").unwrap(),
					U256::from_dec_str("16397242299165439412854345647920988068174648012394612865878728614273133142679").unwrap(),
				],
				vec![
					U256::from_dec_str("17800981185981546727325645880080997001476675047091692601614288140235437092669").unwrap(),
					U256::from_dec_str("18740407875764131523921983143028060498186857273067309944248989237432327688403").unwrap(),
				]
			],
			vec![
				vec![
					U256::from_dec_str("665772934717175177777949524098312092352764855175335277424591571425164110495").unwrap(),
					U256::from_dec_str("14100679580899773878692255597131271645919787770156674267632120179255213163919").unwrap(),
				],
				vec![
					U256::from_dec_str("11550483577636357067438501129303480300849727408381097797290690228588830430753").unwrap(),
					U256::from_dec_str("9393054484305485081338275095253982915344196959843349099876609724972300123416").unwrap(),
				],
			],
			vec![
				vec![
					U256::from_dec_str("11935857310771353301113078485816547646734693035868999119793358992399371905384").unwrap(),
					U256::from_dec_str("11076391388262632214871501512549398943625460082535158648764826100124319684977").unwrap(),
				],
				vec![
					U256::from_dec_str("3710665449666757495807260984495481142659978308600359337780598549499120138938").unwrap(),
					U256::from_dec_str("6839803279250166998041675738233682206460875086384455306323331018124376381750").unwrap(),
				],
				vec![
					U256::from_dec_str("4856493623582275736770069967742171908738373534012202112313355761651931717892").unwrap(),
					U256::from_dec_str("7519753620411063893993038654245337200396081075176594821170650629333484307735").unwrap(),
				],
				vec![
					U256::from_dec_str("3710770660542853524064560607689078015597423835131235506678560299694689990300").unwrap(),
					U256::from_dec_str("5614292083398031256628051490208894298153846073526157332595481788509741490501").unwrap(),
				],
				vec![
					U256::from_dec_str("20078380866648795629722839999599227955238325032514828038799728064893941679517").unwrap(),
					U256::from_dec_str("20119759053067063735852536902576059766817546016509338827246433947870754928064").unwrap(),
				],
				vec![
					U256::from_dec_str("21534411071117043370935210470042510482349669599514544586993218380262317946766").unwrap(),
					U256::from_dec_str("3732652593424164815218097269577235093879112219070020480532471157884566435575").unwrap(),
				],
				vec![
					U256::from_dec_str("914202501664188624437217460875067356655116871746027158611430275383721262460").unwrap(),
					U256::from_dec_str("4705734910142038706008803835426807351430196670983074725174783743408516772252").unwrap(),
				],
				vec![
					U256::from_dec_str("13269671382861063801742656963590767807574299083001338650656571090411520374435").unwrap(),
					U256::from_dec_str("9384491173431647196874286778881722649539037427812437964731971759984118937933").unwrap(),
				],
				vec![
					U256::from_dec_str("13125729058979703434326838390930436241715053680615211947758998735780663742234").unwrap(),
					U256::from_dec_str("18251916520687748821686727257519897597261855663648932489281707717905152117467").unwrap(),
				],
				vec![
					U256::from_dec_str("17344976452759912859216359556893576140899470724959684738889968775015869557197").unwrap(),
					U256::from_dec_str("6389133661768713273368557048511335405494395213461472689525395186324704423376").unwrap(),
				],
				vec![
					U256::from_dec_str("6654285101199080658766691402142671572613283476585253631584625231199984295297").unwrap(),
					U256::from_dec_str("8781098977168486108004226572214388660098607817298006388220592119281389247107").unwrap(),
				],
				vec![
					U256::from_dec_str("6222451681253934476137669768565736742025192662359478248756258775339796690138").unwrap(),
					U256::from_dec_str("10135462584425053010794649955169082355399546688339847053640822820061877996779").unwrap(),
				],
				vec![
					U256::from_dec_str("9001863072157687892542090021143207450374263146576957968961232315660797205691").unwrap(),
					U256::from_dec_str("15485950372480454815754011467338385709915066127940363063996809472444868794768").unwrap(),
				],
				vec![
					U256::from_dec_str("13854609252443964014253500266170779699772110475010273610098934711691037018431").unwrap(),
					U256::from_dec_str("20183951798722484000813923295692960832600657304539510159073478598858845049884").unwrap(),
				],
				vec![
					U256::from_dec_str("5009149762864565156931790177584262364524677499527751998450184844302995178615").unwrap(),
					U256::from_dec_str("13593924634624192524816933676404605700054878909664226093265862736105894857387").unwrap(),
				],
				vec![
					U256::from_dec_str("3235679020196617835617599419401376052093999618129660419896130076627172691776").unwrap(),
					U256::from_dec_str("3672945914965606232288988588985362074692638978894414972844938524443558943294").unwrap(),
				],
				vec![
					U256::from_dec_str("19997430478065411692446626965613571768401892436308376031777419505234898610525").unwrap(),
					U256::from_dec_str("19496028539659666741307055192005464165365294110953457722351854025470690326368").unwrap(),
				],
				vec![
					U256::from_dec_str("1126032053729725776969471166249327076697801974223436266234319829936142994428").unwrap(),
					U256::from_dec_str("1936782966048939191038918746223733360522372518808305064770434481582538990769").unwrap(),
				],
				vec![
					U256::from_dec_str("13322112226503241766242224434970603478241980167632482340846160572506467551993").unwrap(),
					U256::from_dec_str("8183177369271153094538982832407066557058612152657745323415828015061401611878").unwrap(),
				],
				vec![
					U256::from_dec_str("20914993233890112480061138799883565180987813697290385292901393950679298583773").unwrap(),
					U256::from_dec_str("2158104559326662347973637565816542707843557115605024872122369365555774497928").unwrap(),
				],
				vec![
					U256::from_dec_str("17919815233747636707024612684230423799222935350810031519005389277454122478732").unwrap(),
					U256::from_dec_str("11388966857843853598028901823801313703810538258166813391542959146274971387057").unwrap(),
				],
			]
		);

		let valid_input = vec![
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
			U256::from_dec_str("100000").unwrap(),
		];

		let verifying_key = VerifyingKey::new(proof, vk, valid_input);

		let zk_verifier = ZKPrecompileVerifier::new(
			H256::from_slice(&sp_io::hashing::keccak_256(b"verify(uint256[2],uint256[2][2],uint256[2],uint256[2],uint256[2][2],uint256[2][2],uint256[2][2],uint256[2][],uint256[])")[0..32]),
			verifying_key
		);

		let encoded_call = zk_verifier.generate_benchmarking_parameters();

		let gas_limit_call = 1000000;
		let value = U256::default();
		let is_transactional = true;
		let validate = true;
	}:{
		let call_runner_results = <T as pallet_evm::Config>::Runner::call(
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
			<T as pallet_evm::Config>::config(),
		);
		match call_runner_results {
			Ok(info) => {
				let output = info.value;
				let used_gas = info.used_gas;
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"copy_20 output result {:?} used_gas: {:?}",
					output,
					used_gas,
				);
				assert!(output.len() >= 32, "The contract did not return true");
				let mut result_bytes = [0u8; 32];
				result_bytes.copy_from_slice(&output[output.len() - 32..]);
				let result = U256::from_big_endian(&result_bytes);
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"copy_20 Verification result {result:?}",
				// );
				assert_eq!(result, U256::one(), "The contract did not return true");
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"copy_20 Benchmarking failed",
				);
				panic!("copy_20 Benchmarking failed");
			}
		}
	}

}
impl_benchmark_test_suite!(
	Pallet,
	crate::zk_precompile_gas_estimation::tests::new_test_ext(),
	crate::zk_precompile_gas_estimation::mock::Test
);
