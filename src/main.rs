pub mod network;
pub mod cutekoi;
pub mod block;

use crate::cutekoi::{spawn_miner, Transaction};
use block::block::Block;
use network::UdpBroadcast;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use rand::random;

fn main() {
    let (data_sender, data_receiver) = channel();
    let (block_sender, block_receiver) = channel::<Block>();
    let (cancel_sender, cancel_receiver) = channel::<()>();
    let zero_length = 8;

    let client_id: usize = random();
    println!("Client ID: {}", client_id);

    let network = UdpBroadcast::new().unwrap();

    spawn_miner(block_sender, network.chain.clone(), cancel_receiver, data_receiver, zero_length);
    network.spawn(block_receiver, cancel_sender);

    loop {
        let amount = random();
        let transaction = Transaction {
            sender: client_id,
            amount,
        };
        let transaction = vec![transaction];
        data_sender.send(transaction).unwrap();
    }
}