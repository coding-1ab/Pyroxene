use crate::block::Address;


pub struct Transaction{
    to: Address,
    from: Address,
    value: u128,
    nonce: u64,
    verifier: Address,
}