use crate::cutekoi::{spawn_miner, Transaction};
use crate::network::UdpBroadcast;
use crate::block::block::Block;

use rand::random;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

pub struct Client{
    pub id: usize,
    pub network: UdpBroadcast,
    pub data_sender: Sender<Vec<Transaction>>,
    data_receiver: Option<Receiver<Vec<Transaction>>>,
    block_receiver: Option<Receiver<Block>>,
    cancel_sender: Option<Sender<()>>,
    cancel_receiver: Option<Receiver<()>>,
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

        let mut client = Self {
            id,
            network,
            data_sender,
            data_receiver: Some(data_receiver),
            block_receiver: Some(block_receiver),
            cancel_sender: Some(cancel_sender),
            cancel_receiver: Some(cancel_receiver),
        };

        client.start(block_sender);

        client
    }

    fn start(&mut self, block_sender: Sender<Block>) {
        let zero_length = 8;

        let data_receiver = self.data_receiver.take().unwrap();
        let block_receiver = self.block_receiver.take().unwrap();
        let cancel_receiver = self.cancel_receiver.take().unwrap();
        let cancel_sender = self.cancel_sender.take().unwrap();

        // miner
        spawn_miner(
            block_sender,
            self.network.chain.clone(),
            cancel_receiver,
            data_receiver,
            zero_length,
        );

        // network
        self.network.spawn(
            block_receiver,
            cancel_sender,
        );
    }
}