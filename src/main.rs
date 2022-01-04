#[macro_use]
extern crate log;

mod remote_conn_svc;
mod packet;

fn main() {
    remote_conn_svc::server(); // move to tokio task eventually
}
