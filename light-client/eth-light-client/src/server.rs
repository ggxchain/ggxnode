use ethers::{abi::AbiEncode, types::H256};
use eyre::Result;
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::json;

use crate::config::Config;
use crate::db::DB;
use crate::merkle;

#[derive(Deserialize)]
struct VerifyReq {
    indices: Vec<u32>,
    hashes: Vec<H256>,
}

#[handler]
async fn status() -> &'static str {
    "ok"
}

#[handler]
async fn root(dep: &mut Depot) -> String {
    let db = dep.obtain::<DB>().expect("get DB");
    let logs = db.select_logs().expect("get logs");
    let hashes = logs
        .iter()
        .flat_map(|log| log.transaction_hash)
        .collect::<Vec<_>>();

    json!({
        "root": merkle::root(&hashes).encode_hex()
    })
    .to_string()
}

#[handler]
async fn verify(req: &mut Request, dep: &mut Depot) -> String {
    let verify_req = req
        .parse_body::<VerifyReq>()
        .await
        .expect("Could not parse VerifyReq");
    let db = dep.obtain::<DB>().expect("get DB");
    let logs = db.select_logs().expect("get logs");
    let hashes = logs
        .iter()
        .flat_map(|log| log.transaction_hash)
        .collect::<Vec<_>>();

    let verified =
        merkle::verify(&hashes, &verify_req.indices, &verify_req.hashes).expect("Could not verify");

    json!({ "verified": verified }).to_string()
}

pub async fn start_server(config: Config, db: DB) -> Result<()> {
    let host_and_port = format!("127.0.0.1:{}", config.server_port.unwrap_or(5800));
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
