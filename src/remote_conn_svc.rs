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
    pub async fn run(mut self) -> Result<()> {
        let server_addr = "127.0.0.1:6007".parse().unwrap();
        let (server_config, _server_cert) = self.configure_server().unwrap();
        let (endpoint, mut incoming) = Endpoint::server(server_config, server_addr)?;
        println!("remote_conn_svc running on {}", endpoint.local_addr()?);

        while let Some(conn) = incoming.next().await {
            info!("remote client connecting from {}", conn.remote_address());
            let fut = self.handle_connection(conn);
            if let Err(e) = fut.await {
                error!("connection failed: {}", e.to_string())
            }
        }

        Ok(())
    }

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
            .max_concurrent_uni_streams(0_u8.into())
            .keep_alive_interval(std::time::Duration::new(5,0).into());

        Ok((server_config, cert_der))
    }

    async fn handle_connection(&mut self, conn: quinn::Connecting) -> Result<()> {
        let quinn::NewConnection {connection: _, mut bi_streams, ..} = conn.await?;

        async {
            println!("established");
            while let Some(stream) = bi_streams.next().await {
                let stream = match stream {
                    Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                        println!("connection closed");
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

    async fn handle_request(&mut self, (mut send, recv): (quinn::SendStream, quinn::RecvStream)) -> Result<()> {
        let req = recv.read_to_end(64 * 1024).await.map_err(|e| anyhow!("failed reading request: {}", e))?;
        let pkt = decode(req);
        let resp = self.process_packet(pkt).await;/*.unwrap_or_else(|e| {
            error!("failed: {}", e);
            format!("failed to process request: {}\n", e).into_bytes()
        });*/

        send.write_all(&resp.unwrap()).await.map_err(|e| anyhow!("failed to send response: {}", e))?;
        send.finish().await.map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;
        info!("complete!");

        Ok(())
    }

    async fn process_packet(&mut self, mut pkt: Packet) -> Result<Vec<u8>> {
        self.tx.send(pkt).await;
        let resp = self.rx.recv().await;
        pkt = resp.unwrap();
        pkt.timestamp = std::time::SystemTime::now();
        println!("{:?}", pkt.payload);
        let res = encode(pkt);
    
        Ok(res)
    }
}
