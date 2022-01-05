// adapted from quinn example code
use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use quinn::{Endpoint, ServerConfig};
use std::{error::Error, sync::Arc};
use tracing::{error, info};
use tokio::sync::{mpsc::Receiver, watch::Sender};

use super::packet::*;

pub async fn server(rx: Receiver<()>, tx: Sender<()>) -> Result<()> {
    let code = {
        if let Err(e) = run(rx, tx).await {
            eprintln!("ERROR: {}", e);
            1
        } else {
            0
        }
    };
    ::std::process::exit(code);
}

pub async fn run(rx: Receiver<()>, tx: Sender<()>) -> Result<()> {
    let server_addr = "127.0.0.1:6007".parse().unwrap();

    let (server_config, _server_cert) = configure_server().unwrap();
    let (endpoint, mut incoming) = Endpoint::server(server_config, server_addr)?;
    println!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = incoming.next().await {
        info!("connection incoming");
        let fut = handle_connection(conn);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                error!("connection failed: {}", e.to_string())
            }
        });
    }

    Ok(())
}

#[allow(clippy::field_reassign_with_default)]
fn configure_server() -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(0_u8.into());
    
    Ok((server_config, cert_der))
}

async fn handle_connection(conn: quinn::Connecting) -> Result<()> {
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
            let fut = handle_request(stream);
            tokio::spawn(
                async move {
                    if let Err(e) = fut.await {
                        error!("failed: {}", e.to_string());
                    }
                }
            );
        }
        Ok(())
    }.await?;
    Ok(())
}

async fn handle_request((mut send, recv): (quinn::SendStream, quinn::RecvStream)) -> Result<()> {
    let req = recv.read_to_end(64 * 1024).await.map_err(|e| anyhow!("failed reading request: {}", e))?;
    let pkt = decode(req);
    let resp = process_packet(pkt).unwrap_or_else(|e| {
        error!("failed: {}", e);
        format!("failed to process request: {}\n", e).into_bytes()
    });

    send.write_all(&resp).await.map_err(|e| anyhow!("failed to send response: {}", e))?;
    send.finish().await.map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;
    info!("complete!");

    Ok(())
}

fn process_packet(mut pkt: Packet) -> Result<Vec<u8>> {
    pkt.timestamp = std::time::SystemTime::now();
    println!("{:?}", pkt.payload);
    let res = encode(pkt);

    Ok(res)
}
