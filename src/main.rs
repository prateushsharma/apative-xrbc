use anyhow:: Result;
use tracing::info;
use tracing_subscriber::{fmt,EnvFilter};

#[derive(Debug)]
struct Config {
    node_id: u32,
    listen_addr: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        let node_id: u32 = std::env::var("NODE_ID")?.parse()?;
        let listen_addr = std::env::var("LISTEN")?;
        Ok(Self { node_id, listen_addr})
    }
}

fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env()
    .add_directive("info".parse()?);
    fmt().with_env_filter(filter).init();

    let cfg = Config::from_env()?;

    info!("Node started with:");
    info!(" ID = {}",cfg.node_id);
    info!(" Listening = {}",cfg.listen_addr);

    Ok(())
}