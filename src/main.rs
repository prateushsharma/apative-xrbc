mod network;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{fmt,EnvFilter};
use network::{Network, Msg};



#[derive(Debug)]
struct Config {
    node_id: u32,
    listen_addr: String,
     peer: String,
}

impl Config {
    fn from_env() -> Result<Self> {
        let node_id: u32 = std::env::var("NODE_ID")?.parse()?;
        let listen_addr = std::env::var("LISTEN")?;
         let peer = std::env::var("PEER")?;
        Ok(Self { node_id, listen_addr, peer })
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::new("info");
    fmt().with_env_filter(filter).init();

    let cfg = Config::from_env()?;

     info!("Node {} starting on {}", cfg.node_id, cfg.listen_addr);

      let net = Network::new(cfg.node_id, cfg.listen_addr.clone());

    tokio::spawn(async move {
        net.run().await.unwrap();
    });

    loop {
        let msg = Msg::Ping { from: cfg.node_id };
        Network::new(cfg.node_id, cfg.listen_addr.clone())
        .send(&cfg.peer, msg)
        .await?;

         tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

   

}