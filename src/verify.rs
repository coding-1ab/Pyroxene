/// 비밀키 공개키 받아서 저장을 하고 어떤 파일을 사인하고 검증하기
///

use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use rsa::pkcs8::{LineEnding};
use rsa::pss::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, Verifier};
use rsa::RsaPrivateKey;
use sha2::Sha256;

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
    let signed_message = key.sign_with_rng(&mut rand::rng(), data);
    signed_message
}

pub fn verify_data(data: &[u8], signature: &Signature, key: VerifyingKey<Sha256>) {
    key.verify(data, signature).expect("Verification failed");
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
