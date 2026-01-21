pub mod network;
pub mod verify;
pub mod block;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use block::block::Block;
use verify::{Transaction, mine};
use network::UdpBroadcast;

fn main() {
    let (block_sender, block_receiver) = channel::<Block>();
    let (cancel_sender, cancel_receiver) = channel::<()>();
    let chain = Arc::new(Mutex::new(Vec::<Block>::new()));
    let transactions = Vec::<Transaction>::new();
    let zero_length = 4;

    let network = UdpBroadcast::new().unwrap();

    thread::scope(|scope| {
        let chain_clone = chain.clone();
        scope.spawn(move || {
            loop {
                if let Some(block) = mine(&transactions, chain_clone.clone(), &cancel_receiver, zero_length) {
                    block_sender.send(block).unwrap();
                }
            }
        });

        network.start(block_receiver, cancel_sender);
    });
}