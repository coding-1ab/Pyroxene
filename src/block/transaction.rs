use rkyv::{Archive, Deserialize, Serialize};

use crate::block::Address;
use chrono::Utc;

#[derive(Archive,Serialize,Deserialize,Debug,Clone)]
#[rkyv(derive(Debug))]
pub struct Transaction{
    to: Address,
    from: Address,
    value: u128,
    pub nonce: u64,
    pub verifier: Address,
    timestamp: i64,
}

impl Transaction{
    pub fn new(
        to: Address,
        from: Address,
        val: u128,
        verfier: Address,
    ) -> Transaction{
        Transaction{
            to: to,
            from: from,
            value: val,
            nonce: 0,
            verifier: verfier,
            timestamp: Utc::now().timestamp()
        }
    }
}