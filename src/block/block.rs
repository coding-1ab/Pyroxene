use rkyv::{Archive, Deserialize, Serialize};
use crate::block::transaction::Transaction;

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub prev_hash: [u8; 32],
    pub id: u64,
    pub nonce: u64,
    pub merkle_root: [u8; 32],
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub block_header: BlockHeader,
    pub txs: Vec<crate::verify::Transaction>,
}
