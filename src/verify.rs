/// 비밀키 공개키 받아서 저장을 하고 어떤 파일을 사인하고 검증하기
///

use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use rsa::pkcs8::{LineEnding};
use rsa::pss::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, Verifier};
use rsa::RsaPrivateKey;
use sha2::{Digest, Sha256};
use rkyv::{Archive, Serialize, Deserialize, to_bytes, rancor::Error};
use crate::block::block::Block;

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub amount: u64,
}

pub fn generate_keys() -> RsaPrivateKey {
    let private_key = RsaPrivateKey::new(
        &mut rand::thread_rng(),
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
    key.sign_with_rng(&mut rand::thread_rng(), data)
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
