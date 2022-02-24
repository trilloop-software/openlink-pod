use anyhow::Result;
use std::sync::Arc;
use tokio::{spawn, sync::{mpsc, Mutex}};

#[macro_use]
mod macros;

mod auth_svc;
mod device;
mod link_svc;
mod packet;
mod remote_conn_svc;
mod emerg_svc;
mod pod_conn_svc;
mod ctrl_svc;
use packet::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create control signals to communicate between services

    // auth-remote
    let (tx_auth_to_remote, rx_auth_to_remote) = mpsc::channel::<Packet>(32);
    let (tx_remote_to_auth, rx_remote_to_auth) = mpsc::channel::<Packet>(32);

    // auth-link
    let (tx_auth_to_link, rx_auth_to_link) = mpsc::channel::<Packet>(32);
    let (tx_link_to_auth, rx_link_to_auth) = mpsc::channel::<Packet>(32);

    // ctrl-emerg
    let (tx_ctrl_to_emerg, rx_ctrl_to_emerg) = mpsc::channel::<Packet>(32);
    let (tx_emerg_to_ctrl, rx_emerg_to_ctrl) = mpsc::channel::<Packet>(32);

    // auth-emerg
    let (tx_auth_to_emerg, rx_auth_to_emerg) = mpsc::channel::<Packet>(32);
    let (tx_emerg_to_auth, rx_emerg_to_auth) = mpsc::channel::<Packet>(32);

    // auth-ctrl
    let (tx_auth_to_ctrl, rx_auth_to_ctrl) = mpsc::channel::<Packet>(32);
    let (tx_ctrl_to_auth, rx_ctrl_to_auth) = mpsc::channel::<Packet>(32);

    // ctrl-pod
    let (tx_ctrl_to_pod, rx_ctrl_to_pod) = mpsc::channel::<u8>(32);
    let (tx_pod_to_ctrl, rx_pod_to_ctrl) = mpsc::channel::<u8>(32);

    // link-pod
    let (tx_link_to_pod, rx_link_to_pod) = mpsc::channel::<u8>(32);
    let (tx_pod_to_link, rx_pod_to_link) = mpsc::channel::<u8>(32);

    // shared memory
    let device_list: Vec<device::Device> = Vec::new();
    let device_list = Arc::new(Mutex::new(device_list));
    let pod_state = Arc::new(Mutex::new(pod_conn_svc::PodState::Unlocked));

    // Create services with necessary control signals
    let auth_svc = auth_svc::AuthSvc {
        rx_remote: rx_remote_to_auth,
        tx_remote: tx_auth_to_remote,

        rx_link: rx_link_to_auth,
        tx_link: tx_auth_to_link,

        rx_pod_status: rx_ctrl_to_auth,
        tx_pod_status: tx_auth_to_ctrl,

        rx_emerg: rx_emerg_to_auth,
        tx_emerg: tx_auth_to_emerg,
    };

    let emerg_svc = emerg_svc::EmergSvc {
        rx_auth: rx_auth_to_emerg,
        tx_auth: tx_emerg_to_auth,

        rx_pod_status: rx_ctrl_to_emerg,
        tx_pod_status: tx_emerg_to_ctrl
    };

    let ctrl_svc = ctrl_svc::CtrlSvc { 
        pod_state: Arc::clone(&pod_state),

        rx_auth: rx_auth_to_ctrl, 
        tx_auth: tx_ctrl_to_auth,

        rx_emerg: rx_emerg_to_ctrl,
        tx_emerg: tx_ctrl_to_emerg,

        rx_pod: rx_pod_to_ctrl,
        tx_pod: tx_ctrl_to_pod
    };

    let link_svc = link_svc::LinkSvc { 
        device_list: Arc::clone(&device_list),
        rx_auth: rx_auth_to_link,
        tx_auth: tx_link_to_auth,
        rx_pod: rx_pod_to_link,
        tx_pod: tx_link_to_pod,
    };

    let remote_conn_svc = remote_conn_svc::RemoteConnSvc { 
        rx: rx_auth_to_remote, 
        tx: tx_remote_to_auth
    };

    let pod_conn_svc = pod_conn_svc::PodConnSvc {
        conn_list: Vec::new(),
        device_list: Arc::clone(&device_list),
        pod_state: pod_state,
        rx_ctrl: rx_ctrl_to_pod,
        tx_ctrl: tx_pod_to_ctrl,
        rx_link: rx_link_to_pod,
        tx_link: tx_pod_to_link,
        //rx_tele: todo!(),
        //tx_tele: todo!(),
    };

    // Spawn all services as tasks
    spawn(auth_svc.run());
    spawn(link_svc.run());
    spawn(remote_conn_svc.run());
    spawn(emerg_svc.run());
    spawn(ctrl_svc.run());
    spawn(pod_conn_svc.run());

    loop {

    }
}
