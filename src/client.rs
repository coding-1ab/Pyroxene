use crate::cutekoi::{spawn_miner, Transaction};
use crate::network::UdpBroadcast;
use crate::block::block::Block;

use rand::random;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;

pub struct Client{
    pub id: usize,
    pub network: UdpBroadcast,
    pub miner_handle: JoinHandle<()>,
    pub network_handle: JoinHandle<()>,
}

impl Client{
    pub fn new() -> Self{
        let (data_sender, data_receiver) = channel();
        let (block_sender, block_receiver) = channel::<Block>();
        let (cancel_sender, cancel_receiver) = channel::<()>();

        let zero_length = 8;
        let id: usize = random();

        println!("Client ID: {}", id);

        let network = UdpBroadcast::new().unwrap();

        let miner_handle = spawn_miner(block_sender, network.chain.clone(), cancel_receiver, data_receiver, zero_length);
        let network_handle = network.spawn(block_receiver, cancel_sender);
    }
}