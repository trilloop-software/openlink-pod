use anyhow::Result;
use std::sync::Arc;
use tokio::{
    spawn,
    sync::{mpsc, Mutex},
};

#[macro_use]
mod macros;

mod auth_svc;
mod ctrl_svc;
mod database_svc;
mod emerg_svc;
mod link_svc;
mod pod_conn_svc;
mod pod_packet;
mod pod_packet_payload;
mod remote_conn_svc;
mod tele_svc;
mod trip_svc;
mod user;

use pod_packet::*;
use shared::{device::*, launch::LaunchParams, remote_conn_packet::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create control signals to communicate between services

    // auth-remote
    let (tx_auth_to_remote, rx_auth_to_remote) = mpsc::channel::<RemotePacket>(32);
    let (tx_remote_to_auth, rx_remote_to_auth) = mpsc::channel::<RemotePacket>(32);

    // auth-link
    let (tx_auth_to_link, rx_auth_to_link) = mpsc::channel::<RemotePacket>(32);
    let (tx_link_to_auth, rx_link_to_auth) = mpsc::channel::<RemotePacket>(32);

    // auth-ctrl
    let (tx_auth_to_ctrl, rx_auth_to_ctrl) = mpsc::channel::<RemotePacket>(32);
    let (tx_ctrl_to_auth, rx_ctrl_to_auth) = mpsc::channel::<RemotePacket>(32);

    // remote-emerg (only one channel needed because nothing is being sent back to client)
    let (tx_remote_to_emerg, rx_remote_to_emerg) = mpsc::channel::<u8>(32);

    // ctrl-pod
    let (tx_ctrl_to_pod, rx_ctrl_to_pod) = mpsc::channel::<PodPacket>(32);
    let (tx_pod_to_ctrl, rx_pod_to_ctrl) = mpsc::channel::<PodPacket>(32);

    // emerg-pod
    let (tx_pod_to_emerg, rx_pod_to_emerg) = mpsc::channel::<u8>(32);
    let (tx_emerg_to_pod, rx_emerg_to_pod) = mpsc::channel::<u8>(32);

    // link-pod
    let (tx_link_to_pod, rx_link_to_pod) = mpsc::channel::<PodPacket>(32);
    let (tx_pod_to_link, rx_pod_to_link) = mpsc::channel::<PodPacket>(32);

    // auth-data
    let (tx_auth_to_data, rx_auth_to_data) = mpsc::channel::<RemotePacket>(32);
    let (tx_data_to_auth, rx_data_to_auth) = mpsc::channel::<RemotePacket>(32);

    // auth-tele
    let (tx_auth_to_tele, rx_auth_to_tele) = mpsc::channel::<RemotePacket>(32);
    let (tx_tele_to_auth, rx_tele_to_auth) = mpsc::channel::<RemotePacket>(32);

    // ctrl-trip
    let (tx_ctrl_to_trip, rx_ctrl_to_trip) = mpsc::channel::<LaunchParams>(32);

    // trip-pod
    let (tx_trip_to_pod, rx_trip_to_pod) = mpsc::channel::<u8>(32);

    // shared memory
    let device_list: Vec<Device> = Vec::new();
    let device_list = Arc::new(Mutex::new(device_list));
    let launch_params = LaunchParams {
        distance: None,
        max_speed: None,
    };
    let pod_state = Arc::new(Mutex::new(pod_conn_svc::PodState::Unlocked));

    // Create services with necessary control signals
    let auth_svc = auth_svc::AuthSvc {
        rx_remote: rx_remote_to_auth,
        tx_remote: tx_auth_to_remote,

        rx_link: rx_link_to_auth,
        tx_link: tx_auth_to_link,

        rx_ctrl: rx_ctrl_to_auth,
        tx_ctrl: tx_auth_to_ctrl,

        rx_data: rx_data_to_auth,
        tx_data: tx_auth_to_data,

        rx_tele: rx_tele_to_auth,
        tx_tele: tx_auth_to_tele,
    };

    let emerg_svc = emerg_svc::EmergSvc {
        rx_pod: rx_pod_to_emerg,
        tx_pod: tx_emerg_to_pod,

        rx_remote: rx_remote_to_emerg,
    };

    let ctrl_svc = ctrl_svc::CtrlSvc {
        launch_params: launch_params,
        pod_state: Arc::clone(&pod_state),

        rx_auth: rx_auth_to_ctrl,
        tx_auth: tx_ctrl_to_auth,

        rx_pod: rx_pod_to_ctrl,
        tx_pod: tx_ctrl_to_pod,

        tx_trip: tx_ctrl_to_trip,
    };

    let link_svc = link_svc::LinkSvc {
        device_list: Arc::clone(&device_list),
        pod_state: Arc::clone(&pod_state),
        rx_auth: rx_auth_to_link,
        tx_auth: tx_link_to_auth,
        rx_pod: rx_pod_to_link,
        tx_pod: tx_link_to_pod,
    };

    let remote_conn_svc = remote_conn_svc::RemoteConnSvc {
        rx_auth: rx_auth_to_remote,
        tx_auth: tx_remote_to_auth,
        tx_emerg: tx_remote_to_emerg,
    };

    let pod_conn_svc = pod_conn_svc::PodConnSvc {
        conn_list: Vec::new(),
        device_list: Arc::clone(&device_list),
        pod_state: Arc::clone(&pod_state),
        rx_ctrl: rx_ctrl_to_pod,
        tx_ctrl: tx_pod_to_ctrl,
        rx_emerg: rx_emerg_to_pod,
        tx_emerg: tx_pod_to_emerg,
        rx_link: rx_link_to_pod,
        tx_link: tx_pod_to_link,
        //rx_tele: todo!(),
        //tx_tele: todo!(),
        rx_trip: rx_trip_to_pod,
    };

    let tele_svc = tele_svc::TelemetrySvc {
        pod_state: Arc::clone(&pod_state),
        tele_data: Vec::new(),
        rx_auth: rx_auth_to_tele,
        tx_auth: tx_tele_to_auth,
    };

    let database_svc = database_svc::DatabaseSvc {
        rx_auth: rx_auth_to_data,
        tx_auth: tx_data_to_auth,
    };

    let trip_svc = trip_svc::TripSvc {
        pod_state: Arc::clone(&pod_state),
        rx_ctrl: rx_ctrl_to_trip,
        tx_pod: tx_trip_to_pod,
    };

    // Spawn all services as tasks
    spawn(auth_svc.run());
    spawn(link_svc.run());
    spawn(remote_conn_svc.run());
    spawn(emerg_svc.run());
    spawn(ctrl_svc.run());
    spawn(pod_conn_svc.run());
    spawn(tele_svc.run());
    spawn(database_svc.run());
    spawn(trip_svc.run());

    loop {}
}
