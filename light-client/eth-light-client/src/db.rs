use std::sync::{Arc, Mutex};

use ethers::types::Log;
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

	pub fn create_table(&self) -> Result<usize> {
		let conn = self.conn.lock().expect("acquire mutex");
		Ok(conn.execute(
			"CREATE TABLE IF NOT EXISTS logs (
				block_number INTEGER NOT NULL,
				log_index INTEGER NOT NULL,
				log TEXT NOT NULL,
				PRIMARY KEY (block_number, log_index)
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

	pub fn select_logs(&self, block_number: u64) -> Result<Vec<Log>> {
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
}
