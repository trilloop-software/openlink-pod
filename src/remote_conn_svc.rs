// adapted from quinn example code
use anyhow::Result;
use futures_util::stream::StreamExt;
use quinn::{Endpoint, ServerConfig};
use std::{error::Error, sync::Arc};
use tracing::{error, info};
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

use super::packet::*;

pub struct RemoteConnSvc {
    pub rx_auth: Receiver<Packet>,
    pub tx_auth: Sender<Packet>,
    pub tx_emerg: Sender<u8>,
}

impl RemoteConnSvc {
    /// Main service function for remote_conn_svc
    /// Open UDP socket and start listening for QUIC connections
    pub async fn run(mut self) -> Result<()> {
        let server_addr = "127.0.0.1:6007".parse().unwrap();
        let (server_config, _server_cert) = self.configure_server().unwrap();
        let (endpoint, mut incoming) = Endpoint::server(server_config, server_addr)?;
        println!("remote_conn_svc: service running on {}", endpoint.local_addr()?);

        while let Some(conn) = incoming.next().await {
            println!("remove_conn_svc: remote client connecting from {}", conn.remote_address());
            let fut = self.handle_connection(conn);
            if let Err(e) = fut.await {
                error!("remote_conn_svc: connection failed: {}", e.to_string());
            }

            println!("remote_conn_svc: remote client closed");
            if let Err(e) = self.tx_emerg.send(1).await {
                eprintln!("remote->emerg failed: {}", e)
            }
            // connection closed, trigger emerg_svc to stop pod if PodState::Moving
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
            .max_idle_timeout(Some(std::time::Duration::from_millis(100).try_into()?)) // 100ms timeout
            .keep_alive_interval(std::time::Duration::from_millis(50).into()); // 50ms heartbeat

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
        let req = match recv.read_to_end(64 * 1024).await {
            Ok(req) => req,
            Err(e) => encode(Packet::new(0, vec![s!(e)]))
        };

        let pkt = decode(req);
        let resp: Vec<u8>;

        if pkt.cmd_type == 0 {
            resp = encode(pkt);
        } else {
            resp = self.process_packet(pkt).await.unwrap();
        }

        match send.write_all(&resp).await {
            Ok(()) => (),
            Err(e) => println!("remote_conn_svc: failed to send response: {}", s!(e))
        }

        match send.finish().await {
            Ok(()) => (),
            Err(e) => println!("remote_conn_svc: failed to shutdown stream: {}", s!(e))
        }

        Ok(())
    }

    /// Send the packet to the auth service
    /// Receive the result from the auth service and update timestamp
    /// If request to auth_svc errored, return the error as the payload and update timestamp
    /// Return packet as buffer
    async fn process_packet(&mut self, pkt: Packet) -> Result<Vec<u8>> {
        let pkt = match self.tx_auth.send(pkt).await {
            Ok(()) => {
                let resp = self.rx_auth.recv().await.unwrap();
                Packet::new(resp.cmd_type, resp.payload)
            },
            Err(e) => {
                Packet::new(0, vec![s!(e)])
            }
        };
        
        let res = encode(pkt);
    
        Ok(res)
    }
}
