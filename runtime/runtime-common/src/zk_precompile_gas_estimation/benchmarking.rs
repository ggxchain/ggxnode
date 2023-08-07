// #![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
// frontier
use pallet_evm::Runner;
use sp_std::prelude::*;

struct VerifyingKey {
	proof_a: Vec<U256>,
	proof_b: Vec<Vec<U256>>,
	proof_c: Vec<U256>,
	vk_alpha: Vec<U256>,
	vk_beta: Vec<Vec<U256>>,
	vk_gamma: Vec<Vec<U256>>,
	vk_delta: Vec<Vec<U256>>,
	vk_ic: Vec<Vec<U256>>,
	valid_input: Vec<U256>,
}

benchmarks! {
	nconstraints_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("323654832133639084097738656057999026789295280666886852594984576709163539526").unwrap(),
			U256::from_dec_str("6142957218280645039826855630872622613337777761157366378615236951849913775542").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("3781402603457401622553078625624300084337446058890012151530042647448923283156").unwrap(),
				U256::from_dec_str("3517436796495769146106898128956048233729888622957356948216773257280783319311").unwrap()
			],
			vec![
				U256::from_dec_str("18697717875318192535466611481750571094853863458428844875498420533596881568336").unwrap(),
				U256::from_dec_str("21426423909981695530725752097567643188095093768132522705564475372400085021436").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("21613404883582191576526635676329374200679201344239432919502376530553941450391").unwrap(),
			U256::from_dec_str("15157755445966041962464884593780030869511182260822371637833098723763718514455").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("11521346682463921906610271064518766485088278761420928386309550940286650671457").unwrap(),
			U256::from_dec_str("14352830518175561380348856554041231805603495989169432000725584250110291361678").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("15782596638536803844844408481637582589736413871396993858917786753770876529739").unwrap(),
				U256::from_dec_str("14911823231521430202833448929652297782496660742577343858219844871404504144036").unwrap()
			],
			vec![
				U256::from_dec_str("2082222914940990020573243578958386229340150899029559516350605378857473031525").unwrap(),
				U256::from_dec_str("13498996195230132373906915048494917069165130851394909752977335076805840989307").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("3598021131780341035246011324427156192540414244243113294885735792953741606880").unwrap(),
				U256::from_dec_str("2036084172531515709709771052774738245922046202318890723044728246225539963428").unwrap()
			],
			vec![
				U256::from_dec_str("5925457711548235099025126547368719549508643015841982506039250719340224619945").unwrap(),
				U256::from_dec_str("6297446610567432982066083461547400065680646594595441716191878482607632761242").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("8540596677770285018018336727251534074846909511956041125063939097422779191925").unwrap(),
				U256::from_dec_str("4004145476643750015352094786522189637304633549022019774555222068038038169997").unwrap()
			],
			vec![
				U256::from_dec_str("14044576757821380530542904664046256148829410228513509672845139471539338485558").unwrap(),
				U256::from_dec_str("21655834928283484760531472827487408029775629615266614212974138202212982461041").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("18160015955331442477546361309657533914485780296451278252095683998154112390415").unwrap(),
				U256::from_dec_str("2421052745427024367257863480713357899822722660532800486395996070904226714086").unwrap()
			],
			vec![
				U256::from_dec_str("13418845437340247256566377432948869602549774753220761674399899795516131627829").unwrap(),
				U256::from_dec_str("19073971532401628571532840417056837528958340897410372352356543204393028153748").unwrap()
			]
		];

		let valid_input = vec![U256::from_dec_str("14965631224775206224").unwrap(), U256::from_dec_str("3021577815302938909").unwrap(), U256::from_dec_str("14359293880404272991").unwrap(), U256::from_dec_str("1555005537055779113").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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

	nconstraints_15 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("20130649533606132941076532567185186533385627048731412173183395399598706653722").unwrap(),
			U256::from_dec_str("5013302449315195110728085938922390324181635865643940685717581319642064255185").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("182349185274378562733946998188496702865899889969489490693307112376287220894").unwrap(),
				U256::from_dec_str("21774901779074752042085216201094185472133753187176013828342990679080664340226").unwrap()
			],
			vec![
				U256::from_dec_str("5869802804905785526619333457887737025688889410113776710579029089235110697192").unwrap(),
				U256::from_dec_str("11299229682492002119651745673740140812409219814076852306943005451224291551902").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("20185206171191904498488721605780661105586137132655998116453037531151235420138").unwrap(),
			U256::from_dec_str("10384110586589833558642758431697087208638469907021113041447281049306264412338").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("16518131354665416356366619968057793725998398101013238576312579236126088696774").unwrap(),
			U256::from_dec_str("16092603482186593319494034095089381923094130901686564523346715807012185857162").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("21734430847137516428081497368042335428479760179384429208483228211390870007889").unwrap(),
				U256::from_dec_str("4340802082106264411360501616317193563940246579398950954214582760982827315863").unwrap()
			],
			vec![
				U256::from_dec_str("12447957849620237825007220112503702667357790011364771059885655868800342557822").unwrap(),
				U256::from_dec_str("14809511833945059264202963765646411224042099188929021476112339598771762711323").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("17324242037467178377673475424723060403816978313062414185741167494411363117422").unwrap(),
				U256::from_dec_str("573707969764376568308084515022276929250394811599970857666617570109713369226").unwrap()
			],
			vec![
				U256::from_dec_str("7128237379235615126712368859451728796195891617721045758701935689078823845047").unwrap(),
				U256::from_dec_str("10251648689582024203541816575587863025500039856770502839005077409839562057857").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("10606414414773769335220870560841720522962914580715128022037427553027383133468").unwrap(),
				U256::from_dec_str("17945431494760095084161450708936397005964902805999383045150147654106302962212").unwrap()
			],
			vec![
				U256::from_dec_str("1114602509780071710197893478088881134916626713904515339987079398421518390581").unwrap(),
				U256::from_dec_str("193998821630871240239454478188468337135142017574640889316476264339168416109").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("5962671215665825419481336630213008800754405882212434260146565927291026766869").unwrap(),
				U256::from_dec_str("16048419286366094768895756097008850316935017795185761633869316473249446959923").unwrap()
			],
			vec![
				U256::from_dec_str("11963041201807212594527101484518323233884176930563963184262271167850375002702").unwrap(),
				U256::from_dec_str("18844805358753225197609605824296109138813209675865830312094534406203255607198").unwrap()
			]
		];

		let valid_input = vec![U256::from_dec_str("4843011250846255110").unwrap(), U256::from_dec_str("10798190704713744688").unwrap(), U256::from_dec_str("15134859568936326220").unwrap(), U256::from_dec_str("2883369245646660076").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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

	nconstraints_20 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("797572477248804392444831225209327368031406989322029802514696832026672358416").unwrap(),
			U256::from_dec_str("14792745011262875092811677474030034668956937211525097875362241962916137456465").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("18780733901893439739371423627306821236863790649657196488891599355591321953851").unwrap(),
				U256::from_dec_str("9680544448661229130288510239162388428669756867621972396627016600055984349499").unwrap()
			],
			vec![
				U256::from_dec_str("11738424367558729478935071422176167853312257211377899850934753533414170335039").unwrap(),
				U256::from_dec_str("18233646049353711985788868148390262624363849777798470342741694587137854046312").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("9030949957263413847331466570610168053394322783722896104311457102193711447736").unwrap(),
			U256::from_dec_str("19892969453275240241999306433024784295865139436513729478470715581670537973183").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("9129483983788792790250516546077504592262306584249196914035593903220113466346").unwrap(),
			U256::from_dec_str("12931577676488623470369386127145659839850514961607631368866187840816419893745").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("7733336395661433750318564428962011373303863005056937588991328321699559429718").unwrap(),
				U256::from_dec_str("17750049529131446238696663224077924496537231636880056629697436601439393528008").unwrap()
			],
			vec![
				U256::from_dec_str("8293377799807146929370692717869590893937274675192938831619975844923042030881").unwrap(),
				U256::from_dec_str("4263357454976975022836583944985361945040552205174286149616049652777546621773").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("14151440830510271657142608578154654007777206023678518530807735001262951045692").unwrap(),
				U256::from_dec_str("1767291767856447136921215746781550838727073342526929959676175537783883225487").unwrap()
			],
			vec![
				U256::from_dec_str("15173492886826553839363776087467612357738824024869668919975796903208737039768").unwrap(),
				U256::from_dec_str("11930052964402267454553402353579104183135244867064411968119095331986593125500").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("7299631212873411609242516274008416712647200249879388590190636109884194725508").unwrap(),
				U256::from_dec_str("8501638231329375259007419224045458732070699435817171076438597961712893321854").unwrap()
			],
			vec![
				U256::from_dec_str("15047730134046003932972688387420458293142941608648562801026918857178341643816").unwrap(),
				U256::from_dec_str("14151346772944327368443846389587994785327280897500259203977403093792866270134").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("731357633257473743905158797156252139612893761758845410915414966970585493895").unwrap(),
				U256::from_dec_str("16967744642853731718627404215453394490485627147408404648214694655394595655398").unwrap()
			],
			vec![
				U256::from_dec_str("1346463672788502974713113436051481443629775449244308207040448867039527497840").unwrap(),
				U256::from_dec_str("18844670641178242764504539209612934748132154582441062514954805312543875749365").unwrap()
			]
		];

		let valid_input = vec![U256::from_dec_str("9545317761180864681").unwrap(), U256::from_dec_str("15203947991842169444").unwrap(), U256::from_dec_str("11010182592065041277").unwrap(), U256::from_dec_str("2750389353565937097").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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

	nconstraints_25 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("5442688768232443344552263783267390208856198689842640798776177410815565547367").unwrap(),
			U256::from_dec_str("16310414003289240410078247718742394211798524980802086932072100478604843138879").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("18190532627346636099188351540696740134404407871692706498689154867172280651496").unwrap(),
				U256::from_dec_str("275967461706136585361355308922764172109936742989119387744117535714873890136").unwrap()
			],
			vec![
				U256::from_dec_str("20432859376686408480939951320620928735684720753051796534902941362107917945448").unwrap(),
				U256::from_dec_str("20575328298763015614805638365408623315084841523436681299021166233260739215585").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("21729070949671190651383446829182792170701603278872739139487134063534373496928").unwrap(),
			U256::from_dec_str("11473159888261500071826283643864569393105028020759142008919378165280126183791").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("6116186011971746741793619516283587796788649022044712022866691859910541408388").unwrap(),
			U256::from_dec_str("9990751801376433017793027493689116570629263722237360070512991124369867111294").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("19134989187129211715471143723472806940780591878437988384412094557596618102949").unwrap(),
				U256::from_dec_str("11002122036813197737012103302067373956587501561568917970913786117384906562322").unwrap()
			],
			vec![
				U256::from_dec_str("20227241808293262878318140546702845934510851943742865848865718895434416271823").unwrap(),
				U256::from_dec_str("1208695332664339284660261403674630632193027591536251936703796527895870746053").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("9988080894620831828514001726353047899833627122523386578651185763598814334161").unwrap(),
				U256::from_dec_str("1210488941280905931352250513800327257407276187767564156715231067181630088303").unwrap()
			],
			vec![
				U256::from_dec_str("9123410895646419307291143661097804061907837645760653809597075433728132210845").unwrap(),
				U256::from_dec_str("5940342572290825895021782196790640411792369989666988944848888929369542627086").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("3845390871684034417765460290906605590268084456131570016989104541941027299587").unwrap(),
				U256::from_dec_str("19451210342848170935105654531273996195376657542644165412726124373232172955450").unwrap()
			],
			vec![
				U256::from_dec_str("12402386430554074306546660621773259523647713153408570111706227556424141104015").unwrap(),
				U256::from_dec_str("12206340396174591766842915594324772300487750563590424709980302152249618278504").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("5288881796355507035555160331041001500160990907930198297197056072823274477221").unwrap(),
				U256::from_dec_str("13632806080609019911417800900671762249376036835109070627111505646509468055118").unwrap()
			],
			vec![
				U256::from_dec_str("17029427908866374993625919576903554103814716934062886672833059297154117520635").unwrap(),
				U256::from_dec_str("4259071212917198769064755546045521459903854045469312364340173149295579006240").unwrap()
			]
		];

		let valid_input = vec![U256::from_dec_str("13322346319448626142").unwrap(), U256::from_dec_str("2125026056706439081").unwrap(), U256::from_dec_str("8305705901596606458").unwrap(), U256::from_dec_str("3095703818812128800").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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

	nconstraints_30 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("18690487469105583830896348523194667998180545629275814901431298749603732361922").unwrap(),
			U256::from_dec_str("7368590978490025622497872418876156479561347771212306366618589942495456841309").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("6458484707522958224040736264962413462988793890751266991780959428701001616723").unwrap(),
				U256::from_dec_str("5229552051142880655000908369897792053786596109840647032497320533202927196186").unwrap()
			],
			vec![
				U256::from_dec_str("14308631389244366765238213169645232719387098057421285153016420565820252114596").unwrap(),
				U256::from_dec_str("5350276954672740804922429900678406990441626154643173671817737280965857560969").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("2116304898851279057226756458603873842331017000957857852682949878590352582212").unwrap(),
			U256::from_dec_str("4667525997409073696628336876173638225578496705097212864335627957772434988886").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("6699928044611798138494735544984517284373216312890158010790742091150243853313").unwrap(),
			U256::from_dec_str("13934807201585568304844420589516599892787718850719460623808296221406392586172").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("14027097432418820340011319545278213614698987547810017425098729501416647686036").unwrap(),
				U256::from_dec_str("11193668485833499271260223875108511371976490160655434826777606292642378752410").unwrap()
			],
			vec![
				U256::from_dec_str("20045214665848771726931611212381067857571766687548405278435917898412573204852").unwrap(),
				U256::from_dec_str("5826006366551155612056429708996305319128773563972295690836601225605275501674").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("17806409976608908367323888887822121188242647630168482212485920174169347027063").unwrap(),
				U256::from_dec_str("8825456074095233176609721582085618649257862500585602911204033640423196905248").unwrap()
			],
			vec![
				U256::from_dec_str("7655945321523385522603417723274574493283664938267073386365908771387240995761").unwrap(),
				U256::from_dec_str("1391423751277804315942138214547478409852807217621791637137051903290468385729").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("3809610967392539750715448081264997038074319458499796020415928204603377523576").unwrap(),
				U256::from_dec_str("21881257755244911884500859203243891779642123470948153619959132693051710001501").unwrap()
			],
			vec![
				U256::from_dec_str("18182668523718172991867827330151318111796082889349461468494480610978916492639").unwrap(),
				U256::from_dec_str("18346603307568004229749993259186056067010854478467318912554191630050104816892").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("18607870993123209852816319805711229553001696602336350378390522478455144860988").unwrap(),
				U256::from_dec_str("15734225289122896815446339472863615677791448850890205504711357677016324894100").unwrap()
			],
			vec![
				U256::from_dec_str("3677354038386268432171152235208754414542237317446575011474517892977310796611").unwrap(),
				U256::from_dec_str("2220614331603226929967110674993611028989932204445910742740951381331567463169").unwrap()
			]
		];

		let valid_input = vec![U256::from_dec_str("14321691860995553260").unwrap(), U256::from_dec_str("7152862679273281751").unwrap(), U256::from_dec_str("12752615512303817990").unwrap(), U256::from_dec_str("1576113262537949146").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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

	nconstraints_n_30_input_10 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("18690487469105583830896348523194667998180545629275814901431298749603732361922").unwrap(),
			U256::from_dec_str("7368590978490025622497872418876156479561347771212306366618589942495456841309").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("6458484707522958224040736264962413462988793890751266991780959428701001616723").unwrap(),
				U256::from_dec_str("5229552051142880655000908369897792053786596109840647032497320533202927196186").unwrap()
			],
			vec![
				U256::from_dec_str("14308631389244366765238213169645232719387098057421285153016420565820252114596").unwrap(),
				U256::from_dec_str("5350276954672740804922429900678406990441626154643173671817737280965857560969").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("2116304898851279057226756458603873842331017000957857852682949878590352582212").unwrap(),
			U256::from_dec_str("4667525997409073696628336876173638225578496705097212864335627957772434988886").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("6699928044611798138494735544984517284373216312890158010790742091150243853313").unwrap(),
			U256::from_dec_str("13934807201585568304844420589516599892787718850719460623808296221406392586172").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("14027097432418820340011319545278213614698987547810017425098729501416647686036").unwrap(),
				U256::from_dec_str("11193668485833499271260223875108511371976490160655434826777606292642378752410").unwrap()
			],
			vec![
				U256::from_dec_str("20045214665848771726931611212381067857571766687548405278435917898412573204852").unwrap(),
				U256::from_dec_str("5826006366551155612056429708996305319128773563972295690836601225605275501674").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("17806409976608908367323888887822121188242647630168482212485920174169347027063").unwrap(),
				U256::from_dec_str("8825456074095233176609721582085618649257862500585602911204033640423196905248").unwrap()
			],
			vec![
				U256::from_dec_str("7655945321523385522603417723274574493283664938267073386365908771387240995761").unwrap(),
				U256::from_dec_str("1391423751277804315942138214547478409852807217621791637137051903290468385729").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("3809610967392539750715448081264997038074319458499796020415928204603377523576").unwrap(),
				U256::from_dec_str("21881257755244911884500859203243891779642123470948153619959132693051710001501").unwrap()
			],
			vec![
				U256::from_dec_str("18182668523718172991867827330151318111796082889349461468494480610978916492639").unwrap(),
				U256::from_dec_str("18346603307568004229749993259186056067010854478467318912554191630050104816892").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("18607870993123209852816319805711229553001696602336350378390522478455144860988").unwrap(),
				U256::from_dec_str("15734225289122896815446339472863615677791448850890205504711357677016324894100").unwrap()
			],
			vec![
				U256::from_dec_str("3677354038386268432171152235208754414542237317446575011474517892977310796611").unwrap(),
				U256::from_dec_str("2220614331603226929967110674993611028989932204445910742740951381331567463169").unwrap()
			]
		];

		let valid_input = vec![U256::from_dec_str("14321691860995553260").unwrap(), U256::from_dec_str("7152862679273281751").unwrap(), U256::from_dec_str("12752615512303817990").unwrap(), U256::from_dec_str("1576113262537949146").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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

	nconstraints_n_30_input_15 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("12887020598237220198032976835022673435562149625889668367844845965730098834830").unwrap(),
			U256::from_dec_str("5547693013562036425753166956109600351181086648379743178595634512552397114069").unwrap()
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("16962228903458078536456881207808446370619165879473741769286461031761302313925").unwrap(),
				U256::from_dec_str("21359014405084746685422343887329959215710120429954270380405915213638462365773").unwrap()
			],
			vec![
				U256::from_dec_str("1965804446739736522247393806555217146673165319190290198953140579215540365682").unwrap(),
				U256::from_dec_str("17336220908557918009722993863856522126371082788415822133704463388052631237429").unwrap()
			],
		];

		let proof_c = vec![
			U256::from_dec_str("21289287840088286145802499082911728754744514711185794471060410933719935352734").unwrap(),
			U256::from_dec_str("7735843223624767963016539689312235639526880990688180630973191899109833388707").unwrap()
		];

		let vk_alpha = vec![
			U256::from_dec_str("8219837847335410476943086063649040035108946850004215058051338864845204653730").unwrap(),
			U256::from_dec_str("3746608922869186366337371343000548817878727551441378244314034669462645204200").unwrap()
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("10791921184212451462910111849505135112215560450850904575300701984074058761885").unwrap(),
				U256::from_dec_str("4316039429358514857629767756887568663426369969101124991106261138164361746936").unwrap()
			],
			vec![
				U256::from_dec_str("4955635812176343518126942275795640701344828035542260039596033477800959201829").unwrap(),
				U256::from_dec_str("18835858855587795231442750838451623232696087390908796571380078611078267640296").unwrap()
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("14455985103474869293499052450161566063643705959843058655267780689179003897023").unwrap(),
				U256::from_dec_str("18671940900597221048350051608932531159222193402258576279517995276727240232360").unwrap()
			],
			vec![
				U256::from_dec_str("12697560912244320596982687035610656622602540075343333727038001021203210410728").unwrap(),
				U256::from_dec_str("18504263146151879520679575960347812434433829897678644699385693906415996014848").unwrap()
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("21037497448680943522446235827846511504661282584273281863975228687729637949136").unwrap(),
				U256::from_dec_str("21776379845891271099314141614451177025469923146966666607913073612769825420476").unwrap()
			],
			vec![
				U256::from_dec_str("11117373909154352545964641618271566801006796957927201839858282655319054162622").unwrap(),
				U256::from_dec_str("8761601020737608627675880810307531774963186960493596493629509853637521319476").unwrap()
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("14615765550099232335848106179629691206556397912266277872446949704093515221045").unwrap(),
				U256::from_dec_str("6162554008291819782928114499558697072224852754500626110745064581632604362032").unwrap()
			],
			vec![
				U256::from_dec_str("8566823584588002931097794273610208557671819171252077099603486344498744585143").unwrap(),
				U256::from_dec_str("10591536333958951217832992999045910076326646568032772769778666286942586222933").unwrap()
			]
		];

		let valid_input = vec![U256::from_dec_str("5692445613459621787").unwrap(), U256::from_dec_str("12028678087606031844").unwrap(), U256::from_dec_str("16634303442742113517").unwrap(), U256::from_dec_str("1607701919348517559").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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

	nconstraints_n_30_input_20 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("675300276299245578591094709236990240708914273209938055982582668214795758174").unwrap(),
			U256::from_dec_str("13990735151356840356168472337981584444541365401592649678306674505655642720744").unwrap(),
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("20662974626567138590845164891104877742207018449180572731972302829490961465377").unwrap(),
				U256::from_dec_str("17980826672123191014823546738817744353214706462261563577840570306372367452611").unwrap(),
			],
			vec![
				U256::from_dec_str("16771588723175231559633339886504703803804691684198872315901738104890051757400").unwrap(),
				U256::from_dec_str("15483086789303204255210084974843463497785089107989053404746931842499188347781").unwrap(),
			],
		];

		let proof_c = vec![
			U256::from_dec_str("10803326508903591698850591270453065951979818405855788885186501914161310127604").unwrap(),
			U256::from_dec_str("7463976916832770397677605389090326852042103199297295653656426503462443575095").unwrap(),
		];

		let vk_alpha = vec![
			U256::from_dec_str("17808795307067734350448942252050504857802750572499920704974512866462124896173").unwrap(),
			U256::from_dec_str("17115681179782086865134005803599891667690289004882667885655096788650956914423").unwrap(),
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("19972461112908818195114127269905083158285147872659880745754440161709753445934").unwrap(),
				U256::from_dec_str("5931352730208730296622877139521064931325621630284477295549537572620163532752").unwrap(),
			],
			vec![
				U256::from_dec_str("2882185593168745381926882623953300896502789920459989016767915325150853635011").unwrap(),
				U256::from_dec_str("18631580128182038958786817732606869739243773104340288087215229087084869651401").unwrap(),
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("851770636198195762853634545486979938053193628863032188824852166527357841269").unwrap(),
				U256::from_dec_str("14031995609625725240579021647119854585086016231593377303976803993039287310339").unwrap(),
			],
			vec![
				U256::from_dec_str("20686783438308398939975162795637102332306903166807161288857973107679921334009").unwrap(),
				U256::from_dec_str("2532011633268289192635445441332602676050528250745931428259769182816495045641").unwrap(),
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("14714684631646051284376146303759912153045379177454388567159304861853969170973").unwrap(),
				U256::from_dec_str("21581087789843023955779607070023575401154216928170460432554645594586663631552").unwrap(),
			],
			vec![
				U256::from_dec_str("18800497152775773104784586380904309935376489820003075987643947940762274526501").unwrap(),
				U256::from_dec_str("17553212126947187473201629145084482836990004337579399572737167549853121474562").unwrap(),
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("7232111346224912559397152061373338385516036432961030742660350233926241518205").unwrap(),
				U256::from_dec_str("1312106886866516986858011448492089642322868115679821300916062234780269911574").unwrap(),
			],
			vec![
				U256::from_dec_str("6433608899686307828382554879645744712090018725772338372746278420606357265684").unwrap(),
				U256::from_dec_str("10454877373084482498589840527780357413752247536960233289477564876826588932778").unwrap(),
			]
		];

		let valid_input = vec![U256::from_dec_str("3912017569823150797").unwrap(), U256::from_dec_str("13734487482202628151").unwrap(), U256::from_dec_str("17621575820406120931").unwrap(), U256::from_dec_str("942862921734396008").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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


	nconstraints_n_30_input_25 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("16006545354898718229260951824688742329434625591330021352594388758149873016896").unwrap(),
			U256::from_dec_str("1468205165503169447069936905882957144235585999244028403117646701884216745392").unwrap(),
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("8852923839895344533619890530783168672405232228640552748869548310905734934401").unwrap(),
				U256::from_dec_str("14205972468980433410756631183566705893026836831976200390499104030484143466072").unwrap(),
			],
			vec![
				U256::from_dec_str("5273488309799458449304345924433747290680464512658479589035286165842771509674").unwrap(),
				U256::from_dec_str("16380658691502717574062613685364050177124879164561079589092985937963750638698").unwrap(),
			],
		];

		let proof_c = vec![
			U256::from_dec_str("20115812596296892999368823778004624226181425602585514202472590919479136110663").unwrap(),
			U256::from_dec_str("4894925383921626127869982825697178632493733742654662142214903567620044651205").unwrap(),
		];

		let vk_alpha = vec![
			U256::from_dec_str("7941376557483595089971660523849374119116282550293913525516084066171760665760").unwrap(),
			U256::from_dec_str("11518059386572304338895027869539849446766258314854264516705855524316523480782").unwrap(),
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("6169389823915190479258473920339684633183862258518826493764760771800535359821").unwrap(),
				U256::from_dec_str("17130452746849022487073859340540869381062009205302178500507024318010199396952").unwrap(),
			],
			vec![
				U256::from_dec_str("2318544008261920780923055770521739382224787976556999331708881512332672347971").unwrap(),
				U256::from_dec_str("1745424225911873055606510145529079726501693479367526857564720869880667302655").unwrap(),
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("623010707232623462659826601801211232554955056016532044807183015392365318482").unwrap(),
				U256::from_dec_str("865232306164050822814381934297685639795963777883253356917760252650841495108").unwrap(),
			],
			vec![
				U256::from_dec_str("20499284403708841372819161133131038743430624942488488352074984316323625475649").unwrap(),
				U256::from_dec_str("8586534605108047515719534665260604577359156625952909070129967685099418328393").unwrap(),
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("17466746742513346286036517376644263104859467956003283191454165897758552595857").unwrap(),
				U256::from_dec_str("5311323701152602220187351314264762414138054616620164066845262834294902529310").unwrap(),
			],
			vec![
				U256::from_dec_str("21683159430467589572447310729564760337034354389144135006135111895877568793948").unwrap(),
				U256::from_dec_str("14137150650205647732181183821072311493301762612748140449230051456997255133827").unwrap(),
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("7587790987123565575798590220561861914433653529800504392689910235318401885521").unwrap(),
				U256::from_dec_str("838359042085515464552622656979816858987026153105984521769778297124070509054").unwrap(),
			],
			vec![
				U256::from_dec_str("13124708479886510248630494255189994749135078654655301928099114054732758024498").unwrap(),
				U256::from_dec_str("10651906791373118981569686350085926733476784541744614206187906526770681605261").unwrap(),
			]
		];

		let valid_input = vec![U256::from_dec_str("1514079710047709751").unwrap(), U256::from_dec_str("9927892042756486157").unwrap(), U256::from_dec_str("6795743742855535984").unwrap(), U256::from_dec_str("3178922015906793710").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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
	nconstraints_n_30_input_30 {
		use frame_benchmarking::vec;
		use sp_core::{H160, U256};
		use ethabi::Token;

		let caller = "d43593c715fdd31c61141abd04a99fd6822c8558".parse::<H160>().unwrap();

		let contract_address = H160::from_low_u64_be(0x8888);

		let proof_a = vec![
			U256::from_dec_str("192383223808332879678223120765873656071487020594757668859947327172062021463").unwrap(),
			U256::from_dec_str("20631800377548143112330602909240361504775196179042765937715471405374872743424").unwrap(),
		];

		let proof_b = vec![
			vec![
				U256::from_dec_str("2386546353880304173552960655436514856580592658425839135191217698540932917221").unwrap(),
				U256::from_dec_str("8483190656320044020998407034485182750694926538145978907506797577826038755937").unwrap(),
			],
			vec![
				U256::from_dec_str("7615954358803953940045634219422206780946884331580120097208882088362520213892").unwrap(),
				U256::from_dec_str("17492892761336449552765644207249662635220549371874833325304908992711325096838").unwrap(),
			],
		];

		let proof_c = vec![
			U256::from_dec_str("14898385768534632459299431757788181862062037231423680879935547707181434062217").unwrap(),
			U256::from_dec_str("3670993976640187257001986589710513145495904997157462001138713422198967083793").unwrap(),
		];

		let vk_alpha = vec![
			U256::from_dec_str("18003389266700451535185768250385473623576088268017984044335576780485413288725").unwrap(),
			U256::from_dec_str("10280152748134343381057553800219778246761591399803713656295141924124651318322").unwrap(),
		];

		let vk_beta = vec![
			vec![
				U256::from_dec_str("15705711023641709606059988467186697257075093473077719904710509531659151640887").unwrap(),
				U256::from_dec_str("13926583045600156240485358993385159153394021178689044035959053205092441158347").unwrap(),
			],
			vec![
				U256::from_dec_str("2133281544711261852175935529948152371794373217468774127814272513506339581229").unwrap(),
				U256::from_dec_str("15266525202021918250529116255077653452497658133580868510244476123771794689087").unwrap(),
			],
		];

		let vk_gamma = vec![
			vec![
				U256::from_dec_str("5860932391264318788101713658077318120931708405538141099278626653654316360162").unwrap(),
				U256::from_dec_str("18777995924606299529855429493114686078258294137554638538976277652307549742619").unwrap(),
			],
			vec![
				U256::from_dec_str("18381097066271335811045821138274998569498024818257561362659949818983456872550").unwrap(),
				U256::from_dec_str("13352938901259724996725843903559730864557166420786660986703127196904357078738").unwrap(),
			],
		];

		let vk_delta = vec![
			vec![
				U256::from_dec_str("14145680588609876894681651972036258026026055123025250786927547400650826638120").unwrap(),
				U256::from_dec_str("11416850029017788620938610876201750823987651931635585676983799773100880269198").unwrap(),
			],
			vec![
				U256::from_dec_str("17147929122794757054114222102821483783601496318354908417750078988687436213430").unwrap(),
				U256::from_dec_str("2750254376147201225382240026981296587762562040528353782979098667468106833906").unwrap(),
			]
		];

		let vk_ic = vec![
			vec![
				U256::from_dec_str("11111422270719484684442669128605095987218378443749504485297022350994995674515").unwrap(),
				U256::from_dec_str("9009256400033220730477021494653362520677702130766423677465372172912927069348").unwrap(),
			],
			vec![
				U256::from_dec_str("711398778459754876097070294398450479822869186578007297816303943615642796091").unwrap(),
				U256::from_dec_str("19953433553885418442641654921645346560623468995060701812216801687443587349442").unwrap(),
			]
		];

		let valid_input = vec![U256::from_dec_str("961725266697111609").unwrap(), U256::from_dec_str("7616088020774934521").unwrap(), U256::from_dec_str("4985267006460356416").unwrap(), U256::from_dec_str("1376874925466056414").unwrap()];
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
					// 	"Verification result {result:?}",
					// );
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
