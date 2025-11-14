use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info,error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Msg {
    Ping { from: u32 },
    Pong { from: u32 },
}

pub struct Network {
    pub node_id: u32,
    pub listen_addr: String,
}

impl Network {
    pub fn new(node_id:u32, listen_addr: String) -> Self {
        Self { node_id, listen_addr}
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("listening on {}", self.listen_addr);

        loop {
            let (mut socket, addr) = listener.accept().await?;
            let node_id = self.node_id;

            tokio::spawn(async move {
                if let Err(e) =Self::handle_connection(node_id, &mut socket).await {
                    error!("connection from {} failed: {:?}", addr, e);
                }
            });
        }
    }

    pub async fn send(&self, addr: &str, msg: Msg) -> Result<()> {
        let mut stream = TcpStream::connect(addr).await?;
        let data = serde_json::to_vec(&msg)?;
        let len = (data.len() as u32).to_be_bytes();

        stream.write_all(&len).await?;
        stream.write_all(&data).await?;

        Ok(())
    }

    async fn handle_connection(node_id: u32, socket: &mut TcpStream) -> Result<()> {
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        socket.read_exact(&mut data).await?;

        let msg: Msg = serde_json::from_slice(&data)?;
        match msg {
            Msg::Ping { from} => {
                info!("Node {} received PING from {}", node_id, from);
            }
            Msg::Pong { from } => {
                info!("Node {} recieved PONG from {}", node_id, from);
            }
        }

        Ok(())
    }



}