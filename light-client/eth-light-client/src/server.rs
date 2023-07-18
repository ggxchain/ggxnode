use ethers::{abi::AbiEncode, types::H256};
use eyre::{Report, Result};
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::json;

use crate::{config::Config, db::DB, merkle};

#[derive(Deserialize)]
struct RootReq {
	block_hash: H256,
}

#[derive(Deserialize)]
struct VerifyReq {
	block_number: u64,
	hashes: Vec<H256>,
	indices: Vec<u32>,
}

#[handler]
async fn status() -> &'static str {
	"ok"
}

#[handler]
async fn root(req: &mut Request, dep: &mut Depot) -> eyre::Result<String> {
	let root_req = req
		.parse_body::<RootReq>()
		.await
		.map_err(|_| Report::msg("Could not parse RootReq"))?;
	let db = dep.obtain::<DB>().ok_or(Report::msg("Could not get DB"))?;
	let receipts = db
		.select_receipts_by_block_hash(root_req.block_hash)
		.map_err(|_| Report::msg("Could not get receipts"))?;
	let hashes = receipts
		.iter()
		.flat_map(|receipt| receipt.logs.iter().flat_map(|log| log.transaction_hash))
		.collect::<Vec<_>>();

	Ok(json!({
		"root": merkle::root(&hashes).encode_hex()
	})
	.to_string())
}

#[handler]
async fn verify(req: &mut Request, dep: &mut Depot) -> Result<String> {
	let verify_req = req
		.parse_body::<VerifyReq>()
		.await
		.map_err(|_| Report::msg("Could not parse VerifyReq"))?;
	let db = dep.obtain::<DB>().ok_or(Report::msg("Could not get DB"))?;
	let logs = db
		.select_logs_by_block_number(verify_req.block_number)
		.map_err(|_| Report::msg("Could not get logs"))?;
	let hashes = logs
		.iter()
		.flat_map(|log| log.transaction_hash)
		.collect::<Vec<_>>();

	let verified = merkle::verify(&hashes, &verify_req.indices, &verify_req.hashes)
		.map_err(|_| Report::msg("Could not verify"))?;

	Ok(json!({ "verified": verified }).to_string())
}

pub async fn start_server(config: Config, db: DB) -> Result<()> {
	let host_and_port = format!(
		"{}:{}",
		config.server_host.unwrap_or("127.0.0.1".to_string()),
		config.server_port.unwrap_or(5800)
	);
	log::info!("server is going to listen {}", host_and_port);

	let router = Router::with_path("/api/v1")
		.push(Router::with_path("status").get(status))
		.push(
			Router::with_path("root")
				.hoop(affix::inject(db.clone()))
				.get(root),
		)
		.push(
			Router::with_path("verify")
				.hoop(affix::inject(db))
				.post(verify),
		);
	let acceptor = TcpListener::new(host_and_port).bind().await;
	Server::new(acceptor).serve(router).await;

	Ok(())
}
