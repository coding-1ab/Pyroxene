use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use rsa::pkcs8::{LineEnding};
use rsa::pss::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, Verifier};
use rsa::RsaPrivateKey;
use sha2::{Digest, Sha256};
use rkyv::{Archive, Serialize, Deserialize, to_bytes, rancor::Error};
use crate::block::block::Block;

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub amount: u64,
}

pub fn generate_keys() -> RsaPrivateKey {
    let private_key = RsaPrivateKey::new(
        &mut rand::rng(),
        2048
    ).expect("Failed to generate a private key");

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
    key.sign_with_rng(&mut rand::rng(), data)
}

pub fn verify_data(data: &[u8], signature: &Signature, key: VerifyingKey<Sha256>) -> bool {
    key.verify(data, signature).is_ok()
}

pub fn hash_block(block: &Block) -> [u8; 32] {
    let bytes = to_bytes::<Error>(block)
        .expect("Failed to serial");

    let hash = Sha256::digest(&bytes);
    hash.into()
}

pub fn verify_chain_link(
    received: &Block,
    local_tip: &Block,
) -> bool {
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

        for pair in level.chunks(2){
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

pub fn mine(transactions: Vec<Transaction>, zero_length: usize) -> Block {
    let prev_hash = [0u8; 32];
    let merkle_root = merkle_root(&transactions);
    let mut nonce = 0u64;

    loop {
        let block = Block {
            block_header: BlockHeader {
                prev_hash,
                nonce,
                merkle_root
            },
            txs: transactions.clone(),
        };

        let hash = hash_block(&block);

        if hash.iter().take(zero_length).all(|&b| b == 0) {
            return block;
        }

        nonce += 1;
    }
}

#[cfg(test)]
mod test {
    use crate::verify::{generate_keys, sign_data, verify_data};
    use rsa::pss::{SigningKey, VerifyingKey};

    #[test]
    fn test_verify() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10u8];
        let key = generate_keys();
        let signature = sign_data(&data, &SigningKey::new(key.clone()));
        verify_data(&data, &signature, VerifyingKey::new(key.to_public_key()));
    }
}
