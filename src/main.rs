mod network;

use anyhow::Result;
use tracing::{info,error};
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
       if let Err(e) =  net.run().await {
        error!("network listener failed: {:?}", e);
       }
    });

    let mut round: u64 = 1;

    loop {
       let payload_str = format!(
        "block proposal from node {} at around {}",
        cfg.node_id, round
       );

       let payload = payload_str.into_bytes();

       let msg = Msg::RbcSend {round, payload};

       if let Err(e)  =Network::send(cfg.node_id, &cfg.peer, msg).await {
        info!("Could not reach peer {}: {:?}", cfg.peer, e);
       }

       round +=1;

       tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }

   

}