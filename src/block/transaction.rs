use rkyv::{Archive, Deserialize, Serialize};

use crate::block::Address;
use chrono::Utc;

#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[rkyv(derive(Debug))]
pub struct Transaction{
    pub to: Address,
    pub from: Address,
    pub value: u128,
    pub nonce: u64,
    pub timestamp: i64,
}

impl Transaction{
    pub fn new(
        to: Address,
        from: Address,
        value: u128,
    ) -> Transaction{
        Transaction{
            to,
            from,
            value,
            nonce: 0,
            timestamp: Utc::now().timestamp()
        }
    }
}