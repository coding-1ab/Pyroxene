
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
    use rkyv::from_bytes;

    #[test]
    fn test_packet_broadcast() {
        let node1 = UdpBroadcast::with_port(1201, 1201).unwrap();
        let node2 = UdpBroadcast::with_port(1201, 1201).unwrap();

        // 보낼 패킷
        let packet = Packet::new(Ipv4Addr::new(192, 168, 0, 100));

        let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&packet).unwrap();

        let mut receive_buffer = [0u8; 1024]; // 1KB 버퍼 재사용

        let receiver = thread::scope(|scope| {
            let result = scope.spawn(|| {
                thread::sleep(Duration::from_millis(100));
                let (size, addr) = node2.recv(&mut receive_buffer).unwrap();
                (size, addr)
            });

            scope.spawn(|| {
                thread::sleep(Duration::from_millis(50));
                node1.send(&serialized).unwrap();
            });

            result.join().unwrap()
        });

        let (received_size, _sender_addr) = receiver;


        let valid_data = &receive_buffer[..received_size];

        let archived = from_bytes::<Packet, rkyv::rancor::Error>(valid_data).unwrap();

        assert_eq!(archived.sender(), packet.sender());
    }
}