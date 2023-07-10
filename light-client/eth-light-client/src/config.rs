use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub consensus_rpc: String,
    pub untrusted_rpc: String,
    pub smart_contract_address: String,
    pub block_number: Option<u64>,
    pub db_path: Option<String>,
    pub helios_home_path: Option<String>,
    pub server_port: Option<u64>,
}
