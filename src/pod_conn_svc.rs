use anyhow::Result;
use std::sync::Arc;
use tokio::{net::TcpStream, sync::{mpsc::Receiver, mpsc::Sender, Mutex}};

// in pod_state_svc?
pub enum PodState {
    Unlocked,
    Locked,
    Moving,
    Braking
}

pub struct PodConnSvc {
    conn_list: Vec<TcpStream>,
    
    pub device_list: Arc<Mutex<Vec<super::device::Device>>>,
    pub pod_state: Arc<Mutex<PodState>>,

    pub rx_ctrl: Receiver<i32>,
    pub tx_ctrl: Sender<i32>,
    pub rx_link: Receiver<u8>,
    pub tx_link: Sender<u8>,
    pub rx_tele: Receiver<i32>,
    pub tx_tele: Sender<i32>,
}

/// pod_conn_svc opens and manages tcp streams to all embedded devices
/// sends all embedded commands
/// receives responses from embedded devices
impl PodConnSvc {
    pub async fn run(mut self) {
        println!("pod_conn_svc: service running");

        loop {
            tokio::select! {
                _ctrl_cmd = self.rx_ctrl.recv() => {
                    self.send_cmd()
                },
                link_cmd = self.rx_link.recv() => {
                    // link_svc lock received, start up all tcp connections
                    // if successful, set pod state to locked state and trigger discovery packet
                    // 
                    // if unsuccessful report unchanged state to user?
                    match link_cmd.unwrap() {
                        1 => {
                            let res = match self.populate_conn_list().await {
                                Ok(()) => 1,
                                Err(()) => 0,
                            };

                            self.tx_link.send(res).await;
                        },
                        _ => ()
                    }
                },
                _tele_cmd = self.rx_tele.recv() => {
                    self.get_telemetry()
                }
            }
        }
    }

    fn get_telemetry(&mut self) {
        // iterate through conn_list TcpStreams
        // send commands to receive all telemetry
        // return to telemetry_svc
    }

    async fn populate_conn_list(&mut self) -> Result<(), ()> {
        if !self.conn_list.is_empty() {
            self.conn_list.clear()
        }

        // create TcpStream for each device in devicelist, and push to conn_list
        for dev in self.device_list.lock().await.clone() {
            let addr = format!("{}:{}", dev.ip_address, dev.port);
            match TcpStream::connect(addr).await {
                Ok(s) => self.conn_list.push(s),
                Err(_) => {}
            }
        }

        // if devicelist.length == conn_list.length then all devices are connected and pod_state is locked
        if self.device_list.lock().await.len() != self.conn_list.len() {
            return Err(())
        }
        *self.pod_state.lock().await = PodState::Locked;

        Ok(())
    }

    fn send_cmd(&mut self) {
        // send command to associated devices
        // (discovery packet should return array of available commands, figure out best way to store this and query it)
    }
}

