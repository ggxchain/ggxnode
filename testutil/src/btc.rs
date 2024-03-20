use testcontainers::{
	core::{Image, WaitFor},
	ImageArgs,
};

use testcontainers::Container;

pub extern crate bitcoincore_rpc;

use bitcoincore_rpc::Auth;

pub use bitcoincore_rpc::Client;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BtcNodeImage {
	image: String,
	tag: String,
}

impl Default for BtcNodeImage {
	fn default() -> Self {
		Self {
			image: "ruimarinho/bitcoin-core".to_string(),
			tag: "22".to_string(),
		}
	}
}

impl BtcNodeImage {
	pub fn with_image(mut self, image: String) -> Self {
		self.image = image;
		self
	}

	pub fn with_tag(mut self, tag: String) -> Self {
		self.tag = tag;
		self
	}
}

impl Image for BtcNodeImage {
	type Args = BtcNodeArgs;

	fn name(&self) -> String {
		self.image.clone()
	}

	fn tag(&self) -> String {
		self.tag.clone()
	}

	// fn expose_ports(&self) -> Vec<u16> {
	// 	vec![
	// 		18443, // rpc
	// 	]
	// }

	fn ready_conditions(&self) -> Vec<WaitFor> {
		vec![WaitFor::message_on_stdout("init message: Done loading")]
	}
}

#[derive(Debug, Clone)]
pub struct BtcNodeArgs {
	args: Vec<String>,
}
impl Default for BtcNodeArgs {
	fn default() -> Self {
		Self {
			args: vec![
				"-regtest",
				"-server",
				"-txindex",
				"-rpcuser=bitcoin",
				"-rpcpassword=bitcoin",
				"-rpcport=18443",
				"-rpcbind=0.0.0.0",
				"-rpcallowip=0.0.0.0/0",
				"-fallbackfee=0.0002",
			]
			.iter()
			.map(|s| s.to_string())
			.collect(),
		}
	}
}

impl ImageArgs for BtcNodeArgs {
	fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
		Box::new(self.args.into_iter())
	}
}

pub struct BtcNodeContainer<'d>(pub Container<'d, BtcNodeImage>);
impl<'d> BtcNodeContainer<'d> {
	pub fn get_rpc_port(&self) -> u16 {
		self.0.get_host_port_ipv4(18443)
	}

	pub fn get_rpc_url(&self) -> String {
		format!("http://{}:{}", self.get_host(), self.get_rpc_port())
	}

	pub fn get_username(&self) -> String {
		"bitcoin".to_string()
	}

	pub fn get_password(&self) -> String {
		"bitcoin".to_string()
	}

	pub fn get_host(&self) -> String {
		"127.0.0.1".to_string()
	}

	pub fn api_with_host_network(&self, url_suffix: Option<&str>) -> Client {
		self.api_with_host_port(url_suffix, "127.0.0.1", 18443)
	}

	pub fn api_with_host_port(&self, url_suffix: Option<&str>, host: &str, port: u16) -> Client {
		let url = format!("http://{host}:{port}/{}", url_suffix.unwrap_or(""));

		Client::new(
			url.as_str(),
			Auth::UserPass(self.get_username(), self.get_password()),
		)
		.expect("Failed to create RPC client")
	}
}

#[cfg(test)]
mod testcontainers_ggx {
	use super::*;
	use bitcoincore_rpc::{bitcoin::Network, RpcApi};
	use testcontainers::clients::Cli;

	#[tokio::test]
	async fn test_btc_node() {
		let docker = Cli::default();
		let image: BtcNodeImage = BtcNodeImage::default();
		let node = BtcNodeContainer(docker.run(image));
		let api = node.api_with_host_network(None);

		// without this we cannot create new address
		api.create_wallet("test", None, None, None, None).unwrap();

		let address = api
			.get_new_address(None, None)
			.expect("Failed to get new address")
			.require_network(Network::Regtest)
			.expect("Should use regtest network");

		// we need to mine 100 blocks to make 1st block spendable
		api.generate_to_address(101, &address).unwrap();
		let balance = api.get_balance(None, None).unwrap();
		assert_eq!(balance.to_btc(), 50.0);
	}
}
