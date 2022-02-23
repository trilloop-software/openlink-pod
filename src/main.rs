use anyhow::Result;
//use std::sync::Arc;
use tokio::{spawn, sync::{/*broadcast,*/ mpsc/*, Mutex*/}};

#[macro_use]
mod macros;

mod auth_svc;
mod device;
mod link_svc;
mod packet;
mod remote_conn_svc;
mod brake_svc;
mod emerg_svc;
mod launch_svc;
mod pod_conn_svc;
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

    // brake-auth
    let (tx_brake_to_auth, rx_brake_to_auth) = mpsc::channel::<Packet>(32);
    let (tx_auth_to_brake, rx_auth_to_brake) = mpsc::channel::<Packet>(32);

    // brake-emerg
    let (tx_brake_to_emerg, rx_brake_to_emerg) = mpsc::channel::<Packet>(32);
    let (tx_emerg_to_brake, rx_emerg_to_brake) = mpsc::channel::<Packet>(32);

    // brake-launch
    let (tx_brake_to_launch, rx_brake_to_launch) = mpsc::channel::<Packet>(32);
    let (tx_launch_to_break, rx_launch_to_break) = mpsc::channel::<Packet>(32);

    // auth-emerg
    let (tx_auth_to_emerg, rx_auth_to_emerg) = mpsc::channel::<Packet>(32);
    let (tx_emerg_to_auth, rx_emerg_to_auth) = mpsc::channel::<Packet>(32);

    // auth-launch
    let (tx_auth_to_launch, rx_auth_to_launch) = mpsc::channel::<Packet>(32);
    let (tx_launch_to_auth, rx_launch_to_auth) = mpsc::channel::<Packet>(32);

    // link-pod
    let (tx_link_to_pod, rx_link_to_pod) = mpsc::channel::<Vec<device::Device>>(32);
    let (tx_pod_to_link, rx_pod_to_link) = mpsc::channel::<Vec<device::Device>>(32);

    // Create services with necessary control signals
    let auth_svc = auth_svc::AuthSvc {
        rx_remote: rx_remote_to_auth,
        tx_remote: tx_auth_to_remote,

        rx_link: rx_link_to_auth,
        tx_link: tx_auth_to_link,

        rx_brake: rx_brake_to_auth,
        tx_brake: tx_auth_to_brake,

        rx_emerg: rx_emerg_to_auth,
        tx_emerg: tx_auth_to_emerg,

        rx_launch: rx_launch_to_auth,
        tx_launch: tx_auth_to_launch
    };

    let brake_svc = brake_svc::BrakeSvc {
        rx_auth: rx_auth_to_brake,
        tx_auth: tx_brake_to_auth,

        rx_emerg: rx_emerg_to_brake, 
        tx_emerg: tx_brake_to_emerg,

        rx_launch: rx_launch_to_break,
        tx_launch: tx_brake_to_launch
    };

    let emerg_svc = emerg_svc::EmergSvc {
        rx_auth: rx_auth_to_emerg,
        tx_auth: tx_emerg_to_auth,

        rx_brake: rx_brake_to_emerg,
        tx_brake: tx_emerg_to_brake
    };

    let launch_svc = launch_svc::LaunchSvc { 
        rx_auth: rx_auth_to_launch, 
        tx_auth: tx_launch_to_auth,

        rx_brake: rx_brake_to_launch,
        tx_brake: tx_launch_to_break,
    };

    let link_svc = link_svc::LinkSvc { 
        device_list: Vec::new(), 
        rx_auth: rx_auth_to_link,
        tx_auth: tx_link_to_auth,
        rx_pod: rx_pod_to_link,
        tx_pod: tx_link_to_pod,
    };

    let remote_conn_svc = remote_conn_svc::RemoteConnSvc { 
        rx: rx_auth_to_remote, 
        tx: tx_remote_to_auth
    };

    // Spawn all services as tasks
    spawn(auth_svc.run());
    spawn(link_svc.run());
    spawn(remote_conn_svc.run());
    spawn(brake_svc.run());
    spawn(emerg_svc.run());
    spawn(launch_svc.run());

    loop {

    }
}
