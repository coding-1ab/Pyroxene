use crate::block::block::Block;
use crate::network::UdpBroadcast;
use crate::utils::{generate_keys, spawn_miner};

use crate::block::transaction::Transaction;
use rand::random;
use std::sync::mpsc::{channel, Receiver, Sender};
use rsa::RsaPrivateKey;

pub struct Client{
    pub id: usize,
    private_key: RsaPrivateKey,
    pub network: UdpBroadcast,
    pub data_sender: Sender<Vec<Transaction>>,
    data_receiver: Receiver<Vec<Transaction>>,
    block_sender: Sender<Block>,
    block_receiver: Receiver<Block>,
    cancel_sender: Sender<()>,
    cancel_receiver: Receiver<()>,
}

impl Client{
    pub fn new() -> Self {
        let (data_sender, data_receiver) = channel();
        let (block_sender, block_receiver) = channel::<Block>();
        let (cancel_sender, cancel_receiver) = channel::<()>();

        let id: usize = random();

        println!("Client ID: {}", id);

        let network = UdpBroadcast::new().unwrap();
        let private_key = generate_keys();

        Self {
            id,
            private_key,
            network,
            data_sender,
            data_receiver,
            block_sender,
            block_receiver,
            cancel_sender,
            cancel_receiver,
        }
    }

    pub fn start(self) {
        let zero_length = 8;

        // miner
        spawn_miner(
            self.block_sender,
            self.network.chain.clone(),
            self.cancel_receiver,
            self.data_receiver,
            zero_length,
        );

        // network
        self.network.spawn(
            self.block_receiver,
            self.cancel_sender,
        );
    }
}