use std::sync::{Arc, Mutex};

use ethers::{
	abi::AbiEncode,
	types::{TransactionReceipt, H256},
};
use eyre::Result;
use helios::prelude::ExecutionBlock;
use rusqlite::Connection;

use crate::config::Config;

#[derive(Clone)]
pub struct DB {
	conn: Arc<Mutex<Connection>>,
}

impl DB {
	pub fn new(config: &Config) -> Result<Self> {
		let conn = if let Some(path) = config.db_path.clone() {
			Connection::open(path)?
		} else {
			Connection::open_in_memory()?
		};

		Ok(DB {
			conn: Arc::new(Mutex::new(conn)),
		})
	}

	pub fn create_tables(&self) -> Result<usize> {
		let conn = self.conn.lock().expect("acquire mutex");
		conn.execute(
			"CREATE TABLE IF NOT EXISTS blocks (
				block_number INTEGER NOT NULL,
				block_hash TEXT NOT NULL,
				block TEXT NOT NULL,
				PRIMARY KEY (block_number, block_hash)
			)",
			(),
		)?;
		Ok(conn.execute(
			"CREATE TABLE IF NOT EXISTS receipts (
				block_hash TEXT NOT NULL,
				receipts TEXT NOT NULL,
				PRIMARY KEY (block_number)
			)",
			(),
		)?)
	}

	pub fn insert_block(&self, block_number: u64, block_hash: H256, block: &str) -> Result<usize> {
		let conn = self.conn.lock().expect("acquire mutex");
		Ok(conn.execute(
			"INSERT INTO blocks(block_number, block_hash, block) values (?1, ?2, ?3)",
			(block_number, block_hash.encode_hex(), block),
		)?)
	}

	pub fn insert_receipts(&self, block_hash: H256, receipts: &str) -> Result<usize> {
		let conn = self.conn.lock().expect("acquire mutex");
		Ok(conn.execute(
			"INSERT INTO receipts(block_hash, receipts) values (?1, ?2)",
			(block_hash.encode_hex(), receipts),
		)?)
	}

	pub fn select_block_by_block_hash(
		&self,
		block_hash: H256,
	) -> Result<Option<ExecutionBlock>> {
		let conn = self.conn.lock().expect("acquire mutex");
		let mut stmt =
			conn.prepare("SELECT block FROM blocks WHERE block_hash = :block_hash")?;
		let raw_blocks_iter = stmt
			.query_map(&[(":block_hash", &block_hash.encode_hex())], |row| {
				row.get::<_, String>(0)
			})?;

		Ok(raw_blocks_iter
			.flatten()
			.flat_map(|raw_blocks| serde_json::from_str(&raw_blocks))
			.collect::<Vec<_>>()
			.get(0)
			.cloned())
	}

	pub fn select_block_by_block_number(
		&self,
		block_number: u64,
	) -> Result<Option<ExecutionBlock>> {
		let conn = self.conn.lock().expect("acquire mutex");
		let mut stmt =
			conn.prepare("SELECT block FROM blocks WHERE block_number = :block_number")?;
		let raw_blocks_iter = stmt
			.query_map(&[(":block_number", &block_number)], |row| {
				row.get::<_, String>(0)
			})?;

		Ok(raw_blocks_iter
			.flatten()
			.flat_map(|raw_blocks| serde_json::from_str(&raw_blocks))
			.collect::<Vec<_>>()
			.get(0)
			.cloned())
	}

	pub fn select_receipts_by_block_hash(
		&self,
		block_hash: H256,
	) -> Result<Vec<TransactionReceipt>> {
		let conn = self.conn.lock().expect("acquire mutex");
		let mut stmt =
			conn.prepare("SELECT receipts FROM receipts WHERE block_hash = :block_hash")?;
		let raw_receipts_iter = stmt
			.query_map(&[(":block_hash", &block_hash.encode_hex())], |row| {
				row.get::<_, String>(0)
			})?;

		Ok(raw_receipts_iter
			.flatten()
			.flat_map(|raw_receipts| serde_json::from_str(&raw_receipts))
			.collect())
	}
}
