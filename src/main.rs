pub mod network;
pub mod cutekoi;
pub mod block;

use crate::cutekoi::spawn_miner;
use block::block::Block;
use cutekoi::Transaction;
use network::UdpBroadcast;
use std::sync::mpsc::channel;

fn main() {
    let (block_sender, block_receiver) = channel::<Block>();
    let (cancel_sender, cancel_receiver) = channel::<()>();
    let transactions = Vec::<Transaction>::new();
    let zero_length = 4;

    let network = UdpBroadcast::new().unwrap();

    spawn_miner(block_sender, network.chain.clone(), cancel_receiver, transactions, zero_length);
    network.start(block_receiver, cancel_sender);
}