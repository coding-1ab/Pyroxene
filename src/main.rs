pub mod network;
pub mod cutekoi;
pub mod block;
pub mod database;
mod client;

use crate::cutekoi::{spawn_miner, Transaction};
use block::block::Block;
use network::UdpBroadcast;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use rand::random;

use client::Client;
fn main() {
    let _client = Client::new();
    loop {
        std::thread::park();
    }
}