use rkyv::{Archive, Deserialize, Serialize};

use crate::block::Address;


#[derive(Archive,Serialize,Deserialize,Debug,Clone)]
#[rkyv(derive(Debug))]
pub struct Transaction{
    to: Address,
    from: Address,
    value: u128,
    nonce: u64,
    verifier: Address,
}

impl Transaction{
    fn new(){

    }
    fn coinbase(){
        
    }
}