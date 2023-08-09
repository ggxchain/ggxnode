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

fn u64s_to_u256(values: Vec<u64>) -> U256 {
	let mut result = U256::zero();
	for (i, value) in values.into_iter().enumerate().take(4) {
		let shift = i * 64;
		result |= U256::from(value) << shift;
	}
	// log::info!(
	// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
	// 	"u64s_to_u256 result {:?}",
	// 	result
	// );
	result
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_10 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_10 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");

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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_50 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_50 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");

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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_1000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_1000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");

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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_900000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_900000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");

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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_30_input_10 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_10 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");

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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_30_input_100 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_100 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_30_input_1000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_1000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_30_input_5000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_5000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_30_input_10000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_10000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");

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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_30_input_90000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_90000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"nconstraints_30_input_200000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"nconstraints_30_input_200000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"sum_10 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_10Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"sum_50 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_50 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"sum_5000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_5000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
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
				// log::info!(
				// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
				// 	"output result {:?}",
				// 	output
				// );
				if output.len() >= 32 {
					let mut result_bytes = [0u8; 32];
					result_bytes.copy_from_slice(&output[output.len() - 32..]);
					let result = U256::from_big_endian(&result_bytes);
					// log::info!(
					// 	target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					// 	"sum_50000 Verification result {result:?}",
					// );
					assert_eq!(result, U256::one(), "The contract did not return true");
				}
			},
			Err(e) => {
				log::info!(
					target: "runtime::runtime-common::zk_precompile_gas_estimation::benchmarking",
					"sum_50000 Benchmarking failed",
				);
				assert!(false, "Benchmarking failed");
			}
		}
	}

}
impl_benchmark_test_suite!(
	Pallet,
	crate::zk_precompile_gas_estimation::tests::new_test_ext(),
	crate::zk_precompile_gas_estimation::mock::Test
);
