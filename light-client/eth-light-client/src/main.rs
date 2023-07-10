use std::sync::{atomic::AtomicBool, Arc};

use eyre::Result;

mod client;
mod config;
mod db;
mod merkle;
mod server;

use client::start_client;
use config::Config;
use db::DB;
use server::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = envy::from_env::<Config>()?;
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    let db = DB::new(&config)?;
    db.create_table()?;

    let clone_db = db.clone();
    let clone_config = config.clone();
    tokio::spawn(async move {
        let res = start_server(clone_config, clone_db).await;
        log::info!("server was stopped, reason: {:?}", res);
    });

    tokio::spawn(async move {
        let res = start_client(config, db.clone(), term).await;
        log::info!("client was stopped, reason: {:?}", res);
    })
    .await
    .expect("stop client successfully");

    Ok(())
}
