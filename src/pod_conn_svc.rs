use anyhow::Result;
use tokio::{net::TcpStream, sync::{mpsc::Receiver, mpsc::Sender}};

// in pod_state_svc?
enum PodState {
    Unlocked,
    Locked,
    Moving,
    Braking
}

pub struct PodConnSvc {
    conn_list: Vec<Option<TcpStream>>,
    pod_state: PodState,

    pub rx_ctrl: Receiver<i32>,
    pub tx_ctrl: Sender<i32>,
    pub rx_link: Receiver<Vec<super::device::Device>>,
    pub tx_link: Sender<Vec<super::device::Device>>,
    pub rx_tele: Receiver<i32>,
    pub tx_tele: Sender<i32>,
}

/// pod_conn_svc opens and manages tcp streams to all 
/// embedded devices
/// sends all embedded commands
/// receives responses from embedded devices
impl PodConnSvc {
    pub async fn run(mut self) {
        println!("pod_conn_svc: service running");

        loop {
            tokio::select! {
                _ctrl_cmd = self.rx_ctrl.recv() => {

                },
                link_cmd = self.rx_link.recv() => {
                    // link_svc lock received, start up all tcp connections
                    // if successful, set pod state to locked state and 
                    // send go ahead to telemetry service, handle discovery packet and return
                    // new device specifications to link_svc
                    // 
                    // if unsuccessful report unchanged state to user?
                    let device_list = link_cmd.unwrap();
                    let device_list = self.populate_conn_list(device_list).unwrap();
                    self.tx_link.send(device_list).await;
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

    fn populate_conn_list(&mut self, device_list: Vec<super::device::Device>) -> Result<Vec<super::device::Device>> {
        if !self.conn_list.is_empty() {
            self.conn_list.clear()
        }

        // create TcpStream for each device in devicelist, and push to conn_list
        // if devicelist.length == conn_list.length then all devices are connected and pod_state is locked
        // send discovery packets to all connected devices
        // update device_list with new connection status, device status, telemetry fields, available commands etc
        // return updated device_list

        Ok(device_list)
    }

    fn send_cmd(&mut self) {
        // send command to associated devices with command available
        // (discovery packet should return array of available commands, figure out best way to store this and query it)
    }
}

