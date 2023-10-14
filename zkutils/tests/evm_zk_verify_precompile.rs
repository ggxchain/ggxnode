use assert_cmd::cargo::cargo_bin;
use ethers::{prelude::*, solc::ProjectPathsConfig};
use std::{
	io::{BufRead, BufReader, Read},
	ops::{Deref, DerefMut},
	process::{Child, Command, Stdio},
};
use tempfile::tempdir;

#[cfg(not(feature = "brooklyn"))]
const CHAIN_ID: u64 = 8886u64;
#[cfg(feature = "brooklyn")]
const CHAIN_ID: u64 = 888866u64;

#[tokio::test]
async fn evm_zk_verify_test1() -> Result<(), Box<dyn std::error::Error>> {
	let base_path = tempdir().expect("could not create a temp dir");

	let cmd = Command::new(cargo_bin("ggxchain-node"))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.args(["--dev"])
		.arg("-d")
		.arg(base_path.path())
		.spawn()?;

	let mut child = KillChildOnDrop(cmd);

	let std_err = child.stderr.take().unwrap();
	let mut buff = BufReader::new(std_err);

	let mut b: [u8; 2048] = [0; 2048];
	buff.read_exact(&mut b)?;
	let v = b.to_vec();

	let (http, _ws) = find_ws_http(&mut v.as_slice())?;

	let temp_dir = tempdir()?;
	let temp_path = temp_dir.path();

	let zk_cmd_out = Command::new(cargo_bin("zkutils"))
		.arg("solidity")
		.arg("verifier")
		.args([
			"tests/circuits/verification_key.json",
			temp_path
				.join("verifier.sol")
				.to_str()
				.ok_or("path join error")?,
		])
		.output()?;
	assert!(zk_cmd_out.status.success());

	let project = Project::builder()
		.paths(
			ProjectPathsConfig::builder()
				.sources(temp_path)
				.artifacts(temp_path.join("build"))
				.cache(temp_path.join("cache"))
				.build()?,
		)
		.build()?;
	let compile_output = project.compile()?;

	let provider: Provider<Http> = Provider::<Http>::try_from(http)?; // Change to correct network
																  // Do not include the private key in plain text in any produciton code. This is just for demonstration purposes
	let wallet: LocalWallet = "0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391"
		.parse::<LocalWallet>()?
		.with_chain_id(CHAIN_ID); // Change to correct network
	let client = SignerMiddleware::new(provider.clone(), wallet.clone());

	let contract = compile_output
		.find_first("Groth16Verifier")
		.ok_or("no contract")?;
	let (abi, bytecode, _) = contract.clone().into_parts();

	let factory = ContractFactory::new(
		abi.ok_or("no abi")?.clone(),
		bytecode.ok_or("no bytecode")?,
		client.clone().into(),
	);
	let deployer = factory.deploy(())?;
	let deployed_contract = deployer.clone().legacy().send().await?;

	abigen!(Groth16Verifier, "tests/circuits/IGroth16Verifier.json",);
	let groth16_verifier = Groth16Verifier::new(deployed_contract.address(), client.clone().into());

	// ["0x27e74cfd62ff9651a900e687aff47585ce893fbdfcec332ef8a7c8f57f1d9df2", "0x2c195dd118f2a838a0098c1ad5d4adb825986afc466a360c4828c8fb766ff48a"],
	// [["0x027f05b1f9b7a5c703e5fb707a6832fa1444c96e5a28ac6a6cda4f9784a7a0cf", "0x0bb52100fd0050a99070d77758cb2022bc41bb0e7791dbfff22f2fb88f751d46"],
	// ["0x2b9ca698dc3d61c83bcd6bf8899a34ea0af8fbfb19f53f37dbe1a4b4f9e0c261", "0x0417d9b66984608d360c7fa22fcd9dc0d29b9adfe5f8c8f5ae68aede69bbcbe9"]],
	// ["0x1a8a474fbdcb2888fa2787bc4425c37d5075f21ccf3a68d704c4173bbec24fa2", "0x2a30a953b5543795c5cb6c995ca862d7a9d1befcc7f304c342a27424528481ed"],
	// ["0x110d778eaf8b8ef7ac10f8ac239a14df0eb292a8d1b71340d527b26301a9ab08"]
	let p_a = [
		U256::from_str_radix(
			"0x27e74cfd62ff9651a900e687aff47585ce893fbdfcec332ef8a7c8f57f1d9df2",
			16,
		)?,
		U256::from_str_radix(
			"0x2c195dd118f2a838a0098c1ad5d4adb825986afc466a360c4828c8fb766ff48a",
			16,
		)?,
	];
	let p_b = [
		[
			U256::from_str_radix(
				"0x027f05b1f9b7a5c703e5fb707a6832fa1444c96e5a28ac6a6cda4f9784a7a0cf",
				16,
			)?,
			U256::from_str_radix(
				"0x0bb52100fd0050a99070d77758cb2022bc41bb0e7791dbfff22f2fb88f751d46",
				16,
			)?,
		],
		[
			U256::from_str_radix(
				"0x2b9ca698dc3d61c83bcd6bf8899a34ea0af8fbfb19f53f37dbe1a4b4f9e0c261",
				16,
			)?,
			U256::from_str_radix(
				"0x0417d9b66984608d360c7fa22fcd9dc0d29b9adfe5f8c8f5ae68aede69bbcbe9",
				16,
			)?,
		],
	];
	let p_c = [
		U256::from_str_radix(
			"0x1a8a474fbdcb2888fa2787bc4425c37d5075f21ccf3a68d704c4173bbec24fa2",
			16,
		)?,
		U256::from_str_radix(
			"0x2a30a953b5543795c5cb6c995ca862d7a9d1befcc7f304c342a27424528481ed",
			16,
		)?,
	];
	let pub_signals = [U256::from_str_radix(
		"0x110d778eaf8b8ef7ac10f8ac239a14df0eb292a8d1b71340d527b26301a9ab08",
		16,
	)?];

	let verify_call = groth16_verifier.verify_proof(p_a, p_b, p_c, pub_signals);
	let valid = verify_call.call().await?;
	assert!(valid, "proof should be valid");

	child.kill()?;
	let _exit_code = child.wait()?;
	Ok(())
}

fn find_ws_http(reader: &mut impl Read) -> Result<(String, String), Box<dyn std::error::Error>> {
	let mut http: Option<String> = None;
	let mut ws: Option<String> = None;

	let buff = BufReader::new(reader);
	let lines = buff.lines().collect::<Result<Vec<String>, _>>()?;

	for line in lines.into_iter() {
		if let Some((_, s)) = line.split_once("Running JSON-RPC HTTP server: addr=") {
			let addr = s.split_once(",").ok_or("failed to split http line")?.0;

			http = Some(format!("http://{}", addr));
		}
		if let Some((_, s)) = line.split_once("Running JSON-RPC WS server: addr=") {
			let addr = s.split_once(",").ok_or("failed to split ws line")?.0;

			ws = Some(format!("ws://{}", addr));
		}

		if http.is_some() && ws.is_some() {
			break;
		}
	}

	Ok((
		http.ok_or("http address not found")?,
		ws.ok_or("ws address not found")?,
	))
}

pub struct KillChildOnDrop(pub Child);

impl Drop for KillChildOnDrop {
	fn drop(&mut self) {
		let _ = self.0.kill();
	}
}

impl Deref for KillChildOnDrop {
	type Target = Child;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for KillChildOnDrop {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
