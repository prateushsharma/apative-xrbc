use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error};
use blake3;

/// Blockchain-style protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Msg {
    // Future use (handshakes etc.)
    Hello { node_id: u32 },

    // Reliable broadcast messages
    RbcSend {
        round: u64,
        payload: Vec<u8>,
    },
    RbcEcho {
        round: u64,
        hash: [u8; 32],
    },
}

/// Network object: owns node identity + listen address
pub struct Network {
    pub node_id: u32,
    pub listen_addr: String,
}

impl Network {
    pub fn new(node_id: u32, listen_addr: String) -> Self {
        Self { node_id, listen_addr }
    }

    /// Start TCP listener, handle inbound connections
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("listening on {}", self.listen_addr);

        loop {
            let (mut socket, addr) = listener.accept().await?;
            let node_id = self.node_id;

            tokio::spawn(async move {
                if let Err(e) = Network::handle_connection(node_id, &mut socket).await {
                    error!("error from {}: {:?}", addr, e);
                }
            });
        }
    }

    /// Send a message to given address, and read a single reply (if any)
    pub async fn send(node_id: u32, addr: &str, msg: Msg) -> Result<()> {
        let mut stream = TcpStream::connect(addr).await?;
        // encode msg
        let data = serde_json::to_vec(&msg)?;
        let len = (data.len() as u32).to_be_bytes();

        // send length + payload
        stream.write_all(&len).await?;
        stream.write_all(&data).await?;

        // try to read a reply (for RbcEcho)
        let mut len_buf = [0u8; 4];
        if let Err(e) = stream.read_exact(&mut len_buf).await {
            // no reply is also okay (peer might just close)
            info!("Node {}: no reply or read error from {}: {:?}", node_id, addr, e);
            return Ok(());
        }
        let reply_len = u32::from_be_bytes(len_buf) as usize;
        let mut reply_buf = vec![0u8; reply_len];
        stream.read_exact(&mut reply_buf).await?;

        let reply_msg: Msg = serde_json::from_slice(&reply_buf)?;
        match reply_msg {
            Msg::RbcEcho { round, hash } => {
                info!(
                    "Node {} received RBC-ECHO from {}: round={}, hash={:?}",
                    node_id,
                    addr,
                    round,
                    hash
                );
            }
            _ => {
                info!(
                    "Node {} got unexpected reply from {}: {:?}",
                    node_id,
                    addr,
                    reply_msg
                );
            }
        }

        Ok(())
    }

    /// Handle 1 inbound message on a connection
    async fn handle_connection(node_id: u32, socket: &mut TcpStream) -> Result<()> {
        // read length prefix
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        socket.read_exact(&mut data).await?;

        let msg: Msg = serde_json::from_slice(&data)?;

        match msg {
            Msg::RbcSend { round, payload } => {
                // this node is acting as RBC receiver
                let h = blake3::hash(&payload);
                let h_bytes: [u8; 32] = *h.as_bytes();

                info!(
                    "Node {} received RBC-SEND: round={}, hash={:?}, size={} bytes",
                    node_id,
                    round,
                    h_bytes,
                    payload.len()
                );

                // reply with RBC-ECHO
                let echo = Msg::RbcEcho { round, hash: h_bytes };
                let encoded = serde_json::to_vec(&echo)?;
                let len = (encoded.len() as u32).to_be_bytes();
                socket.write_all(&len).await?;
                socket.write_all(&encoded).await?;
            }

            Msg::RbcEcho { round, hash } => {
                info!(
                    "Node {} got unexpected RBC-ECHO inbound: round={}, hash={:?}",
                    node_id,
                    round,
                    hash
                );
            }

            Msg::Hello { node_id: other } => {
                info!("Node {} got HELLO from {}", node_id, other);
            }
        }

        Ok(())
    }
}
