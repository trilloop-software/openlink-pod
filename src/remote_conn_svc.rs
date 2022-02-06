// adapted from quinn example code
use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use quinn::{Endpoint, ServerConfig};
use std::{error::Error, sync::Arc};
use tracing::{error, info};
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

use super::packet::*;

pub struct RemoteConnSvc {
    pub rx: Receiver<Packet>,
    pub tx: Sender<Packet>,
}

impl RemoteConnSvc {
    /// Main service function for remote_conn_svc
    /// Open UDP socket and start listening for QUIC connections
    pub async fn run(mut self) -> Result<()> {
        let server_addr = "127.0.0.1:6007".parse().unwrap();
        let (server_config, _server_cert) = self.configure_server().unwrap();
        let (endpoint, mut incoming) = Endpoint::server(server_config, server_addr)?;
        println!("remote_conn_svc running on {}", endpoint.local_addr()?);

        while let Some(conn) = incoming.next().await {
            println!("remote client connecting from {}", conn.remote_address());
            let fut = self.handle_connection(conn);
            if let Err(e) = fut.await {
                error!("connection failed: {}", e.to_string());
            }
        }

        Ok(())
    }

    /// Generates the TLS certificate and other server configurations parameters
    #[allow(clippy::field_reassign_with_default)]
    fn configure_server(&self) -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.serialize_der().unwrap();
        let priv_key = cert.serialize_private_key_der();
        let priv_key = rustls::PrivateKey(priv_key);
        let cert_chain = vec![rustls::Certificate(cert_der.clone())];

        let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
        Arc::get_mut(&mut server_config.transport)
            .unwrap()
            .max_concurrent_uni_streams(0_u8.into())                // force bidirectional streams
            .keep_alive_interval(std::time::Duration::new(5,0).into());    // not necessarily used now but will be useful for future heartbeat functionality

        Ok((server_config, cert_der))
    }

    /// Takes a connecting client and establishes send and receive streams
    async fn handle_connection(&mut self, conn: quinn::Connecting) -> Result<()> {
        let quinn::NewConnection {connection: _, mut bi_streams, ..} = conn.await?;

        async {
            info!("established");
            while let Some(stream) = bi_streams.next().await {
                let stream = match stream {
                    Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                        info!("connection closed");
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(e);
                    }
                    Ok(s) => s,
                };
                let fut = self.handle_request(stream);
                if let Err(e) = fut.await {
                    error!("failed: {}", e.to_string());
                }
            }
            Ok(())
        }.await?;

        Ok(())
    }

    /// Receive request from RecvStream
    /// Decode buffer into valid OpenLink Packet
    /// Send response on SendStream
    async fn handle_request(&mut self, (mut send, recv): (quinn::SendStream, quinn::RecvStream)) -> Result<()> {
        let req = recv.read_to_end(64 * 1024)
            .await
            .map_err(|e| anyhow!("failed reading request: {}", e))?;

        let pkt = decode(req);
        let resp = self.process_packet(pkt).await;

        send.write_all(&resp.unwrap())
            .await
            .map_err(|e| anyhow!("failed to send response: {}", e))?;

        send.finish()
            .await
            .map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;

        info!("complete!");

        Ok(())
    }

    /// Send the packet to the auth service
    /// Receive the result from the auth service and update timestamp
    /// If request to auth_svc errored, return the error as the payload and update timestamp
    /// Return packet as buffer
    async fn process_packet(&mut self, pkt: Packet) -> Result<Vec<u8>> {
        let pkt = match self.tx.send(pkt).await {
            Ok(()) => {
                let resp = self.rx.recv().await;
                let mut pkt = resp.unwrap();
                pkt.timestamp = std::time::SystemTime::now();
                pkt
            },
            Err(e) => {
                let pkt = Packet {
                    packet_id: s!["OPENLINK"],
                    version: 1,
                    cmd_type: 0,
                    timestamp: std::time::SystemTime::now(),
                    payload: vec![s![e]]
                };
                pkt
            }
        };
        
        let res = encode(pkt);
    
        Ok(res)
    }
}
