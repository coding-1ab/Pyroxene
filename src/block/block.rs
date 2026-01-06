use crate::block::transaction::Transaction;

pub struct Block_Header{
    prev_hash: [u8; 32],
    nonce: u64,
    merkle_root: [u8; 32]
}
pub struct Block{
    block_header: Block_Header,
    txs: Vec<Transaction>
}