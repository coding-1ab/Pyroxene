use rkyv::{Archive, Deserialize, Serialize};
use std::net::Ipv4Addr;
use crate::block::block::Block;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(
    compare(PartialEq),
    derive(Debug),
)]
pub struct Packet {
    pub sender_ip: [u8; 4],
}

impl Packet {
    pub fn new(sender: Ipv4Addr) -> Self {
        Self {
            sender_ip: sender.octets(),
        }
    }

    pub fn sender(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.sender_ip)
    }
}

impl ArchivedPacket {
    pub fn sender(&self) -> Ipv4Addr {
        Ipv4Addr::new(
            self.sender_ip[0],
            self.sender_ip[1],
            self.sender_ip[2],
            self.sender_ip[3],
        )
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub enum PacketType {
    NewBlock {
        block: Block,
    },

    ChainLengthRequest,

    BlockRangeRequest {
        start_height: u64,
        end_height: u64,
    },

    ChainLengthResponse {
        chain_length: u64,
    },

    BlockRangeResponse {
        blocks: Vec<Block>,
    },
}

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ProtocolPacket {
    pub sender_ip: [u8; 4],
    pub payload: PacketType,
}

impl ProtocolPacket {
    pub fn new(sender: Ipv4Addr, payload: PacketType) -> Self {
        Self {
            sender_ip: sender.octets(),
            payload,
        }
    }

    pub fn sender(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.sender_ip)
    }

    pub fn packet_type_id(&self) -> u8 {
        match &self.payload {
            PacketType::NewBlock { .. } => 0x01,
            PacketType::ChainLengthRequest => 0x02,
            PacketType::BlockRangeRequest { .. } => 0x03,
            PacketType::ChainLengthResponse { .. } => 0x04,
            PacketType::BlockRangeResponse { .. } => 0x05,
        }
    }

    pub fn is_broadcast(&self) -> bool {
        matches!(
            self.payload,
            PacketType::NewBlock { .. }
                | PacketType::ChainLengthRequest
                | PacketType::BlockRangeRequest { .. }
        )
    }

    pub fn is_response(&self) -> bool {
        matches!(
            self.payload,
            PacketType::ChainLengthResponse { .. }
                | PacketType::BlockRangeResponse { .. }
        )
    }


    pub fn new_block(sender: Ipv4Addr, block: Block) -> Self {
        Self::new(sender, PacketType::NewBlock { block })
    }

    pub fn chain_length_request(sender: Ipv4Addr) -> Self {
        Self::new(sender, PacketType::ChainLengthRequest)
    }

    pub fn block_range_request(sender: Ipv4Addr, start_height: u64, end_height: u64) -> Self {
        Self::new(sender, PacketType::BlockRangeRequest { start_height, end_height })
    }

    pub fn chain_length_response(sender: Ipv4Addr, chain_length: u64) -> Self {
        Self::new(sender, PacketType::ChainLengthResponse { chain_length })
    }

    pub fn block_range_response(sender: Ipv4Addr, blocks: Vec<Block>) -> Self {
        Self::new(sender, PacketType::BlockRangeResponse { blocks })
    }
}

impl ArchivedProtocolPacket {
    pub fn sender(&self) -> Ipv4Addr {
        Ipv4Addr::new(
            self.sender_ip[0],
            self.sender_ip[1],
            self.sender_ip[2],
            self.sender_ip[3],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::block::{Block, BlockHeader};
    use crate::network::UdpBroadcast;
    use std::net::Ipv4Addr;
    use std::thread;
    use std::time::Duration;

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


        let archived = rkyv::access::<ArchivedPacket, rkyv::rancor::Error>(valid_data).unwrap();

        assert_eq!(archived.sender(), packet.sender());
        assert_eq!(archived.sender(), Ipv4Addr::new(192, 168, 0, 100));

        // 역직렬화
        let deserialized: Packet = rkyv::deserialize::<Packet, rkyv::rancor::Error>(archived).unwrap();
        assert_eq!(deserialized, packet);
    }
}