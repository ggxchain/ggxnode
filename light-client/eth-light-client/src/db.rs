use std::sync::{Arc, Mutex};

use ethers::{
	abi::AbiEncode,
	types::{Log, TransactionReceipt, H256},
};
use eyre::Result;
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
			"CREATE TABLE IF NOT EXISTS logs (
				block_number INTEGER NOT NULL,
				log_index INTEGER NOT NULL,
				log TEXT NOT NULL,
				PRIMARY KEY (block_number, log_index)
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

	pub fn insert_logs(&self, block_number: u64, log_index: u64, log: &str) -> Result<usize> {
		let conn = self.conn.lock().expect("acquire mutex");
		Ok(conn.execute(
			"INSERT INTO logs(block_number, log_index, log) values (?1, ?2, ?3)",
			(block_number, log_index, log),
		)?)
	}

	pub fn insert_receipts(&self, block_hash: H256, receipts: &str) -> Result<usize> {
		let conn = self.conn.lock().expect("acquire mutex");
		Ok(conn.execute(
			"INSERT INTO receipts(block_hash, receipts) values (?1, ?2)",
			(block_hash.encode_hex(), receipts),
		)?)
	}

	pub fn select_logs_by_block_number(&self, block_number: u64) -> Result<Vec<Log>> {
		let conn = self.conn.lock().expect("acquire mutex");
		let mut stmt = conn.prepare(
			"SELECT log FROM logs WHERE block_number = :block_number ORDER BY log_index",
		)?;
		let raw_logs_iter = stmt.query_map(&[(":block_number", &block_number)], |row| {
			row.get::<_, String>(0)
		})?;

		Ok(raw_logs_iter
			.flatten()
			.flat_map(|raw_log| serde_json::from_str(&raw_log))
			.collect())
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
