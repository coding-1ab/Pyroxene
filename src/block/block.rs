use sha2::Digest;

use crate::block::transaction::Transaction;


struct BlockHeader {
    prev_hash: [u8; 32],
    nonce: u64,
    merkle_root: [u8; 32]
}
impl BlockHeader{
    
}

pub struct Block{
    block_header: BlockHeader,
    txs: Vec<Transaction>
}

