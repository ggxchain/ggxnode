#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use std::{io::BufReader, process::Command, sync::Arc};
use tempfile::tempdir;

use nix::{
	sys::signal::{kill, Signal::SIGINT},
	unistd::Pid,
};

use std::process::{self};

pub mod common;

use ethers::{
	core::types::U256,
	prelude::*,
	providers::{Http, Provider},
};

#[cfg(not(feature = "testnet"))]
const CHAIN_ID: u64 = 8866u64;
#[cfg(feature = "testnet")]
const CHAIN_ID: u64 = 888866u64;

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

// call precompile zk groth16 verify
async fn call_zk_groth16_verify(
	client: &Client,
	contract_addr: &H160,
) -> Result<(), Box<dyn std::error::Error>> {
	let (
		proof_a,
		proof_b,
		proof_c,
		vk_alpha,
		vk_beta,
		vk_gamma,
		vk_delta,
		vk_ic,
		valid_input,
		invalid_input,
	) = generate_test_case_data()?;

	abigen!(
		ZKGroth16Verify,
		"node/tests/evm_zk_verify_precompile_abi.json",
	);

	// Create contract instance
	let contract = ZKGroth16Verify::new(contract_addr.clone(), Arc::new(client.clone()));

	let valid_tx: bool = contract
		.verify(
			proof_a,
			proof_b,
			proof_c,
			vk_alpha,
			vk_beta,
			vk_gamma,
			vk_delta,
			vk_ic.clone(),
			valid_input,
		)
		.gas(2326400)
		.call()
		.await?;
	println!(
		"Valid Transaction Receipt: {}",
		serde_json::to_string(&valid_tx)?
	);
	assert!(valid_tx);

	let invalid_tx: bool = contract
		.verify(
			proof_a,
			proof_b,
			proof_c,
			vk_alpha,
			vk_beta,
			vk_gamma,
			vk_delta,
			vk_ic,
			invalid_input,
		)
		.gas(2326400)
		.call()
		.await?;
	println!(
		"Invalid Transaction Receipt: {}",
		serde_json::to_string(&invalid_tx)?
	);
	assert!(!invalid_tx);
	Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn evm_zk_verify_test() -> Result<(), Box<dyn std::error::Error>> {
	let base_path = tempdir().expect("could not create a temp dir");

	let mut cmd = Command::new(cargo_bin("ggx-node"))
		.stdout(process::Stdio::piped())
		.stderr(process::Stdio::piped())
		.args(["--dev"])
		.arg("-d")
		.arg(base_path.path())
		.spawn()
		.unwrap();

	let stderr = cmd.stderr.take().unwrap();
	let wrapped = BufReader::new(stderr);

	let (ws_url, http_url, _) = common::find_ws_http_url_from_output(wrapped);

	let mut child = common::KillChildOnDrop(cmd);

	// Let it produce some blocks.
	let _ = common::wait_n_finalized_blocks(1, 30, &ws_url).await;

	assert!(
		child.try_wait().unwrap().is_none(),
		"the process should still be running"
	);

	let provider: Provider<Http> = Provider::<Http>::try_from(http_url)?; // Change to correct network

	// Do not include the private key in plain text in any produciton code. This is just for demonstration purposes
	let wallet: LocalWallet = "0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391"
		.parse::<LocalWallet>()?
		.with_chain_id(CHAIN_ID); // Change to correct network
	let client = SignerMiddleware::new(provider.clone(), wallet.clone());

	let contract_addr = "0x0000000000000000000000000000000000008888".parse::<Address>()?;

	call_zk_groth16_verify(&client, &contract_addr).await?;

	// Stop the process
	kill(Pid::from_raw(child.id() as i32), SIGINT).unwrap();
	child.wait().unwrap();

	Ok(())
}

fn generate_test_case_data() -> Result<
	(
		[U256; 2],
		[[U256; 2]; 2],
		[U256; 2],
		[U256; 2],
		[[U256; 2]; 2],
		[[U256; 2]; 2],
		[[U256; 2]; 2],
		Vec<[U256; 2]>,
		Vec<U256>,
		Vec<U256>,
	),
	Box<dyn std::error::Error>,
> {
	let proof_a = decode_g1_point(
		"13202079600221154376862161785979680082984660469505374274880948735521253479994",
		"19032139815435908179959144311759562497239236177745989139113028703727512477837",
	)?;
	let proof_b = decode_g2_point(
		"9517359327043802798811688827065407805934924568686293993682568334305900037151",
		"13975418982386111217378923290980800393212535787789845393400867460398182717751",
		"11101434469251848949317000686121782094334155840067455941163819739470030872205",
		"3351121397470969456277617123820147601817413346203636355523709813813837616699",
	)?;
	let proof_c = decode_g1_point(
		"21771166379144524714497801611702430117390298454683954881352912868492853507834",
		"5971832614272362565584439633663845994795381011258125087840397908182066694531",
	)?;

	let vk_alpha = decode_g1_point(
		"7318409901911144874440195167086183143676595981815053389579728623121590098440",
		"18845965879715444612950452554360629789407129470518446134938217746489723713219",
	)?;
	let vk_beta = decode_g2_point(
		"4640649673239597789758809808535118135578677216672702870175791505196312738305",
		"13141288066376351908866878766575256664575916601245245304316354941350328880142",
		"11492338667195076401975872253943030431149343004937779351839311477974294172860",
		"17604387530215185597479117283681563543587978658046512716245947218617178983155",
	)?;
	let vk_gamma = decode_g2_point(
		"10857046999023057135944570762232829481370756359578518086990519993285655852781",
		"11559732032986387107991004021392285783925812861821192530917403151452391805634",
		"8495653923123431417604973247489272438418190587263600148770280649306958101930",
		"4082367875863433681332203403145435568316851327593401208105741076214120093531",
	)?;
	let vk_delta = decode_g2_point(
		"5882870888685857628232224840789532289346124290586616915986585508513239272539",
		"8206718104089392401855946495573733123991363841198873660903571227166120193870",
		"14275677868038957349366208693756706908778821863795564855498136614399516409168",
		"20950579407520036072561845357324335488555384097745021047033651867265123837403",
	)?;
	let vk_ic = decode_ic(vec![
		[
			"15329034480187562940265095627808115353397553736992059710948268284574612609224"
				.to_string(),
			"13272704791638435782238987852007128987814629753205340563304933194747762248428"
				.to_string(),
		],
		[
			"17269839325091679315052274785558946544729609490743199699197195008879157661695"
				.to_string(),
			"4142750859697696641705372803120309740931359230261851701215055719438325633654"
				.to_string(),
		],
	])?;

	let valid_input: Vec<U256> = vec![U256::from(66)];
	let invalid_input: Vec<U256> = vec![U256::from(65)];

	Ok((
		proof_a,
		proof_b,
		proof_c,
		vk_alpha,
		vk_beta,
		vk_gamma,
		vk_delta,
		vk_ic,
		valid_input,
		invalid_input,
	))
}

fn decode_g1_point(x: &str, y: &str) -> Result<[U256; 2], Box<dyn std::error::Error>> {
	Ok([U256::from_dec_str(x)?, U256::from_dec_str(y)?])
}

fn decode_g2_point(
	x1: &str,
	x2: &str,
	y1: &str,
	y2: &str,
) -> Result<[[U256; 2]; 2], Box<dyn std::error::Error>> {
	Ok([
		[U256::from_dec_str(x1)?, U256::from_dec_str(x2)?],
		[U256::from_dec_str(y1)?, U256::from_dec_str(y2)?],
	])
}

fn decode_ic(
	points: Vec<[String; 2]>,
) -> Result<Vec<[ethers::types::U256; 2]>, Box<dyn std::error::Error>> {
	let mut result = Vec::new();
	for point in points {
		result.push([
			U256::from_dec_str(&point[0])?,
			U256::from_dec_str(&point[1])?,
		]);
	}
	Ok(result)
}
