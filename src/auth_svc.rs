use anyhow::Result;
use boringauth::pass::is_valid;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::Receiver, mpsc::Sender};

use crate::user::User;
use shared::{login::LoginCredentials, remote_conn_packet::RemotePacket};

const SECRET_KEY: &[u8; 8] = b"openlink";

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize,
    ugroup: u8,
}

pub struct AuthSvc {
    pub rx_remote: Receiver<RemotePacket>,
    pub tx_remote: Sender<RemotePacket>,

    pub rx_link: Receiver<RemotePacket>,
    pub tx_link: Sender<RemotePacket>,

    pub rx_ctrl: Receiver<RemotePacket>,
    pub tx_ctrl: Sender<RemotePacket>,

    pub rx_data: Receiver<RemotePacket>,
    pub tx_data: Sender<RemotePacket>,

    pub rx_tele: Receiver<RemotePacket>,
    pub tx_tele: Sender<RemotePacket>,
}

impl AuthSvc {
    /// Main service task for auth service
    /// Also serves as command parser since all commands require authentication
    pub async fn run(mut self) -> Result<()> {
        println!("auth_svc running");

        while let Some(pkt) = self.rx_remote.recv().await {
            // send packet to associated service based on cmd_type field range
            println!("Packet of type {} received", pkt.cmd_type);
            let resp: RemotePacket = match pkt.cmd_type {
                0..=31 => {
                    // auth service command handling
                    self.auth_handler(&pkt).await.unwrap()
                }
                32..=63 => {
                    // link service command handling
                    // restrict to only admin and software team accounts
                    let ugroup = self.check_token(pkt.token.clone());
                    if ugroup == 0 || ugroup == 1 {
                        RemotePacket::new(0, vec![s!("Not authorized")])
                    } else {
                        if let Err(e) = self.tx_link.send(pkt).await {
                            eprintln!("auth->link failed: {}", e);
                        }
                        self.rx_link.recv().await.unwrap()
                    }
                }
                64..=127 => {
                    // control service command handling
                    // restrict to only admin and mission control accounts
                    let ugroup = self.check_token(pkt.token.clone());
                    if ugroup == 0 || ugroup == 2 {
                        RemotePacket::new(0, vec![s!("Not authorized")])
                    } else {
                        if let Err(e) = self.tx_ctrl.send(pkt).await {
                            eprintln!("auth->launch failed: {}", e);
                        }
                        self.rx_ctrl.recv().await.unwrap()
                    }
                }
                128..=159 => {
                    // telemetry service command handling
                    // all authenticated users can access telemetry
                    if self.check_token(pkt.token.clone()) == 0 {
                        RemotePacket::new(0, vec![s!("Not authorized")])
                    } else {
                        if let Err(e) = self.tx_tele.send(pkt).await {
                            eprintln!("auth->tele failed: {}", e);
                        }
                        self.rx_tele.recv().await.unwrap()
                    }
                }
                160..=195 => {
                    // database service command handling
                    // restrict to only admin accounts
                    if self.check_token(pkt.token.clone()) != 255 {
                        RemotePacket::new(0, vec![s!("Not authorized")])
                    } else {
                        if let Err(e) = self.tx_data.send(pkt).await {
                            eprintln!("auth->database failed: {}", e);
                        }
                        self.rx_data.recv().await.unwrap()
                    }
                }
                196..=227 => {
                    // extra?
                    //
                    pkt
                }
                228..=255 => {
                    // extra?
                    pkt
                }
            };

            // send the modified packet back to remote_conn_svc
            if let Err(e) = self.tx_remote.send(resp).await {
                eprintln!("auth->remote failed: {}", e);
            }
        }

        println!("auth_svc down");

        Ok(())
    }

    /// Handler for auth service defined ranges of cmd_type
    async fn auth_handler(&mut self, pkt: &RemotePacket) -> Result<RemotePacket> {
        let resp = match pkt.cmd_type {
            1 => self.login(&pkt).await,
            _ => RemotePacket::new(0, vec![s!("Command not implemented")]),
        };

        Ok(resp)
    }

    /// Check for user token and return the usergroup
    fn check_token(&self, token: String) -> u8 {
        let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

        match jsonwebtoken::decode::<Claims>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(SECRET_KEY),
            &validation,
        ) {
            Ok(c) => c.claims.ugroup,
            Err(_) => 0,
        }
    }

    /// Generate a token for the user with their usergroup
    fn generate_token(&self, ugroup: u8) -> String {
        let claims = Claims {
            exp: 10000000000,
            ugroup,
        };

        match jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(SECRET_KEY),
        ) {
            Ok(token) => token,
            Err(_) => s!(""),
        }
    }

    /// Query data_svc to check for matching user,
    /// authenticate with boringauth matching hashes of password
    async fn login(&mut self, pkt: &RemotePacket) -> RemotePacket {
        let credentials: LoginCredentials = serde_json::from_str(&pkt.payload[0].clone()).unwrap();
        let user = User::new(credentials.username.clone(), s!("pwd"), 0);
        let user = serde_json::to_string(&user).unwrap();

        if let Err(e) = self.tx_data.send(RemotePacket::new(161, vec![user])).await {
            eprintln!("auth->database failed: {}", e);
        }

        let resp = self.rx_data.recv().await.unwrap();

        match serde_json::from_str::<User>(&resp.payload[0]) {
            Ok(user) => {
                if is_valid(&credentials.password, &user.hash) {
                    RemotePacket::new_with_auth(
                        1,
                        vec![s!("Authenticated"), s!(user.ugroup)],
                        self.generate_token(user.ugroup),
                    )
                } else {
                    if user.name == "" {
                        RemotePacket::new(0, vec![s!("User not found")])
                    } else {
                        RemotePacket::new(0, vec![s!("Wrong password")])
                    }
                }
            }
            Err(_) => RemotePacket::new(0, vec![s!("Login Error")]),
        }
    }
}
