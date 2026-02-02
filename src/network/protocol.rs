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

    pub fn to_bytes(&self) -> Result<Vec<u8>, rkyv::rancor::Error> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self).map(|v| v.into_vec())
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        rkyv::from_bytes::<ProtocolPacket, rkyv::rancor::Error>(data)
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

    #[test]
    fn test_size() {
        let packet1 = ProtocolPacket {
            sender_ip: [0,0,0,0],
            payload: PacketType::ChainLengthRequest,
        };

        let packet2 = ProtocolPacket {
            sender_ip: [0,0,0,0],
            payload: PacketType::BlockRangeRequest {
                start_height: 0,
                end_height: 0,
            },
        };

        let bytes1 = packet1.to_bytes().unwrap();
        let bytes2 = packet2.to_bytes().unwrap();

        assert_ne!(bytes1.len(), bytes2.len())
    }

    #[test]
    fn test_protocol_packet() {
        let block_header = BlockHeader {
            prev_hash: [0u8; 32],
            height: 1,
            nonce: 12345,
            merkle_root: [1u8; 32],
        };
        let block = Block {
            block_header,
            txs: vec![],
        };

        let packet = ProtocolPacket::new_block(Ipv4Addr::new(192, 168, 1, 100), block.clone());

        let bytes = packet.to_bytes().unwrap();
        assert!(bytes.len() > 0);

        let deserialized = ProtocolPacket::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.sender(), Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(deserialized.packet_type_id(), 0x01);

        match deserialized.payload {
            PacketType::NewBlock { block: recv_block } => {
                assert_eq!(recv_block.block_header.height, 1);
                assert_eq!(recv_block.block_header.nonce, 12345);
            }
            _ => panic!("Expected NewBlock packet type"),
        }
    }

    #[test]
    fn test_chain_length_req() {
        let packet = ProtocolPacket::chain_length_request(Ipv4Addr::new(10, 0, 0, 1));

        let bytes = packet.to_bytes().unwrap();
        let deserialized = ProtocolPacket::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.packet_type_id(), 0x02);
        assert!(deserialized.is_broadcast());
        assert!(!deserialized.is_response());
    }

    #[test]
    fn test_block_range_req() {
        let packet = ProtocolPacket::block_range_request(
            Ipv4Addr::new(172, 16, 0, 1),
            10,
            20
        );

        let bytes = packet.to_bytes().unwrap();
        let deserialized = ProtocolPacket::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.packet_type_id(), 0x03);

        match deserialized.payload {
            PacketType::BlockRangeRequest { start_height, end_height } => {
                assert_eq!(start_height, 10);
                assert_eq!(end_height, 20);
            }
            _ => panic!("Expected BlockRangeRequest"),
        }
    }
}