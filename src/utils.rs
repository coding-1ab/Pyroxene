use crate::block::block::{Block, BlockHeader};
use crate::block::transaction::Transaction;
use rkyv::{rancor::Error, to_bytes};
use rsa::pkcs8::LineEnding;
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use rsa::pss::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, Verifier};
use rsa::RsaPrivateKey;
use sha2::{Digest, Sha256};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn generate_keys() -> RsaPrivateKey {
    let private_key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048)
        .expect("Failed to generate a private key");

    private_key
}

pub fn save_key(path: &str, private_key: &RsaPrivateKey) {
    private_key
        .write_pkcs8_pem_file(format!("{}/private.pem", path), LineEnding::CRLF)
        .expect("Failed to write private key");
}

pub fn load_keys(path: &str) -> RsaPrivateKey {
    let private_key = RsaPrivateKey::read_pkcs8_pem_file(format!("{}/private.pem", path)).unwrap();
    private_key
}

pub fn sign_data(data: &[u8], key: &SigningKey<Sha256>) -> Signature {
    key.sign_with_rng(&mut rand::thread_rng(), data)
}

pub fn verify_data(data: &[u8], signature: &Signature, key: VerifyingKey<Sha256>) -> bool {
    key.verify(data, signature).is_ok()
}

pub fn hash_block(block: &Block) -> [u8; 32] {
    let bytes = to_bytes::<Error>(block).expect("Failed to serial");

    let hash = Sha256::digest(&bytes);
    hash.into()
}

pub fn verify_chain_link(received: &Block, local_tip: &Block) -> bool {
    let local_tip_hash = hash_block(local_tip);
    received.block_header.prev_hash == local_tip_hash
}

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(data));
    h
}

fn merkle_root(txs: &[Transaction]) -> [u8; 32] {
    if txs.is_empty() {
        return [0u8; 32];
    }

    let mut level: Vec<[u8; 32]> = txs
        .iter()
        .map(|tx| {
            let bytes = to_bytes::<Error>(tx).unwrap();
            sha256(&bytes)
        })
        .collect();

    while level.len() > 1 {
        let mut next = Vec::new();

        for pair in level.chunks(2) {
            let left = pair[0];
            let right = if pair.len() == 2 { pair[1] } else { pair[0] };

            let mut data = Vec::with_capacity(left.len() + right.len() + 1);
            data.extend_from_slice(&left);
            data.extend_from_slice(&right);

            next.push(sha256(&data));
        }

        level = next;
    }

    level[0]
}

pub fn mine(
    transactions: &Vec<Transaction>,
    chain: Arc<Mutex<Vec<Block>>>,
    control: &Receiver<()>,
    zero_length: usize,
) -> Option<Block> {
    let chain_access = chain.lock().unwrap();
    let prev_hash = chain_access.last().map(hash_block).unwrap_or([0u8; 32]);
    let merkle_root = merkle_root(&transactions);
    let height = chain_access.len() as u64;
    drop(chain_access);

    let mut block = Block {
        block_header: BlockHeader {
            prev_hash,
            height,
            nonce: 0u64,
            merkle_root,
            difficulty: 1.0,
            timestamp: chrono::Utc::now().timestamp(),
        },
        txs: transactions.clone(),
    };

    loop {
        thread::sleep(Duration::from_millis(4));
        if let Ok(_) = control.try_recv() {
            println!("Cancelling Mining");
            return None;
        }

        block.block_header.nonce += 1;

        let hash = hash_block(&block);

        if count_leading_zeros(hash) as usize == zero_length {
            return Some(block);
        }
    }
}

pub fn spawn_miner(
    block_tx: Sender<Block>,
    chain: Arc<Mutex<Vec<Block>>>,
    cancel_rx: Receiver<()>,
    transactions: Receiver<Vec<Transaction>>,
    zero_length: usize,
) {
    thread::spawn(move || {
        loop {
            let to_send = transactions.recv().unwrap();
            loop {
                if let Some(block) = mine(&to_send, chain.clone(), &cancel_rx, zero_length) {
                    block_tx.send(block).unwrap();
                }
            }
        }
    });
}

fn count_leading_zeros(bytes: [u8; 32]) -> u32 {
    let mut count = 0u32;
    for &b in &bytes {
        count += b.leading_zeros();

        if b != 0 {
            break;
        }
    }
    count
}

#[cfg(test)]
mod test {
    use crate::utils::{generate_keys, sign_data, verify_data};
    use rsa::pss::{SigningKey, VerifyingKey};

    #[test]
    fn test_verify() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10u8];
        let key = generate_keys();
        let signature = sign_data(&data, &SigningKey::new(key.clone()));
        verify_data(&data, &signature, VerifyingKey::new(key.to_public_key()));
    }
}
