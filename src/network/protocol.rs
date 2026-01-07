
use rkyv::{Archive, Deserialize, Serialize};

// TODO properly implement this
pub type WalletId = u128;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(
    compare(PartialEq),
    derive(Debug),
)]
pub struct Packet {
    pub sender_id: WalletId,
}

impl Packet {
    pub fn new(sender_id: u128) -> Self {
        Self {
            sender_id
        }
    }

    pub fn sender(&self) -> WalletId {
        self.sender_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::UdpBroadcast;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_packet_broadcast() {
        let node1 = UdpBroadcast::with_port(rand::random::<u128>(), 1201, 1201).unwrap();
        let mut node2 = UdpBroadcast::with_port(rand::random::<u128>(), 1201, 1201).unwrap();

        // 보낼 패킷
        let to_send = Packet::new(node1.id);

        let received = thread::scope(|scope| {
            let result = scope.spawn(|| {
                thread::sleep(Duration::from_millis(100));
                let received = node2.recv().unwrap();
                received
            });

            scope.spawn(|| {
                thread::sleep(Duration::from_millis(50));
                node1.send(&to_send).unwrap();
            });

            result.join().unwrap()
        });

        assert_eq!(to_send, received);
    }
}