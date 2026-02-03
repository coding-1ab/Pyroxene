use rkyv::{Archive, Deserialize, Serialize};

use crate::block::Address;
use chrono::Utc;

#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq)]
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
        value: u128,
        verifier: Address,
    ) -> Transaction{
        Transaction{
            to,
            from,
            value,
            nonce: 0,
            verifier,
            timestamp: Utc::now().timestamp()
        }
    }
}