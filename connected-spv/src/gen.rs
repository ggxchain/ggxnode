use std::path::Path;

use eyre::Result;
use subxt::{error::DispatchError, tx::TxStatus, OnlineClient, PolkadotConfig};
use subxt_signer::{
	bip39::Mnemonic,
	sr25519::{dev, Keypair},
};

#[subxt::subxt(
	runtime_metadata_path = "./metadata/ggxchain-runtime.scale"
	// substitute_type(
	// 	path = "types::primitives::H256",
	// 	with = "::subxt::utils::Static<::types::H256>"
	// )
)]
mod ggxchain {}
