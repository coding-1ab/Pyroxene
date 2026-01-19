use crate::block::transaction::Transaction;

pub struct BlockHeader {
    prev_hash: [u8; 32],
    id: u64,
    nonce: u64,
    merkle_root: [u8; 32]
}
pub struct Block{
    block_header: BlockHeader,
    txs: Vec<Transaction>
}