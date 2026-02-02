use rkyv::{Archive, Deserialize, Serialize};
use chrono::Utc;

use crate::block::transaction::Transaction;
use crate::block::Address;

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct BlockHeader {
    pub prev_hash: [u8; 32],
    pub height: u64,
    pub nonce: u64,
    pub merkle_root: [u8; 32],
    pub difficulty: f32,
    pub timestamp: i64
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct Block {
    pub block_header: BlockHeader,
    pub txs: Vec<Transaction>,
}

impl Block{
    pub fn coinbase(&mut self,to: Address){
        let tx = Transaction::new(to,[0;32], 120, [0;32]);
        if self.txs.len() != 0 {
            panic!("coinbase fuction requires empty block");
        }else {
            self.txs.push(tx);
        }
    }
}