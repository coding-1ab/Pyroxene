use rsa::pss::SigningKey;
use rsa::signature::{Keypair, RandomizedSigner, Verifier};
// 비밀키 공개키 받아서 저장을 하고 어떤 파일을 사인하고 검증하기
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;
use std::fs;
use rsa::pkcs8::{DecodePublicKey, LineEnding};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey};

fn generate_keys(path: &str) {
    let private_key = RsaPrivateKey::new(&mut rand::rng(), 2048).expect("Failed to generate a private key");
    let public_key = RsaPublicKey::from(&private_key);

    private_key.write_pkcs8_pem_file(format!("{}/private.pem", path), LineEnding::CRLF).expect("Failed to write private key");
    public_key.write_public_key_pem_file(format!("{}/public.pem", path), LineEnding::CRLF).expect("Failed to write public key");
}

fn load_keys(path: &str) -> (RsaPrivateKey, RsaPublicKey) {
    let private_key = RsaPrivateKey::read_pkcs8_pem_file(format!("{}/private.pem", path)).unwrap();
    let public_key = RsaPublicKey::read_public_key_pem_file(format!("{}/public.pem",path)).unwrap();

    return (private_key, public_key);
}
fn verify_koi() {
    generate_keys(".");
    let (private_key, public_key) = load_keys(".");

    let text = fs::read("test.txt").unwrap();

    let signing_key = SigningKey::<Sha256>::new(private_key);

    let verifying_key = signing_key.verifying_key();

    let signed_message = signing_key.sign_with_rng(&mut rand::rng(), text.as_slice());
    let check = verifying_key.verify(text.as_slice(), &signed_message).expect("Verification failed");

    println!("Signature check: {:?}", check);
}

#[cfg(test)]
mod test {
    use crate::verify::verify_koi;

    #[test]
    fn testVerify(){
        verify_koi();
    }
}