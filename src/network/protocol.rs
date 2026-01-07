
use rkyv::{Archive, Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(
    compare(PartialEq),
    derive(Debug),
)]
pub struct Packet {
    pub sender_ip: Ipv4Addr,
}

impl Packet {
    pub fn new(sender: Ipv4Addr) -> Self {
        Self {
            sender_ip: sender,
        }
    }

    pub fn sender(&self) -> Ipv4Addr {
        self.sender_ip.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::UdpBroadcast;
    use std::net::Ipv4Addr;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_packet_broadcast() {
        let node1 = UdpBroadcast::with_port(1201, 1201).unwrap();
        let mut node2 = UdpBroadcast::with_port(1201, 1201).unwrap();

        // 보낼 패킷
        let to_send = Packet::new(Ipv4Addr::new(192, 168, 0, 100));

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