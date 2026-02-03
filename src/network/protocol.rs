use crate::block::block::Block;
use rkyv::{Archive, Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Debug)]
pub enum ProtocolError {
    PacketTooShort {
        expected: usize,
        actual: usize,
    },
    UnknownPacketId(u8),
    PayloadTooShort {
        packet_type: &'static str,
        expected: usize,
        actual: usize,
    },
    RkyvError(rkyv::rancor::Error),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PacketTooShort { expected, actual } => write!(
                f,
                "Packet too short: expected at least {} bytes, got {}",
                expected, actual
            ),
            Self::UnknownPacketId(id) => write!(f, "Unknown packet ID: 0x{:02x}", id),
            Self::PayloadTooShort {
                packet_type,
                expected,
                actual,
            } => write!(
                f,
                "{}: expected {} bytes, got {}",
                packet_type, expected, actual
            ),
            Self::RkyvError(e) => write!(f, "rkyv error: {}", e),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<rkyv::rancor::Error> for ProtocolError {
    fn from(e: rkyv::rancor::Error) -> Self {
        Self::RkyvError(e)
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[rkyv(derive(Debug))]
pub enum PacketType {
    NewBlock { block: Block },

    ChainLengthRequest,

    BlockRangeRequest { start_height: u64, end_height: u64 },

    ChainLengthResponse { chain_length: u64 },

    BlockRangeResponse { blocks: Vec<Block> },
}

#[derive(Debug, Clone, PartialEq)]
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
            PacketType::ChainLengthResponse { .. } | PacketType::BlockRangeResponse { .. }
        )
    }

    pub fn new_block(sender: Ipv4Addr, block: Block) -> Self {
        Self::new(sender, PacketType::NewBlock { block })
    }

    pub fn chain_length_request(sender: Ipv4Addr) -> Self {
        Self::new(sender, PacketType::ChainLengthRequest)
    }

    pub fn block_range_request(sender: Ipv4Addr, start_height: u64, end_height: u64) -> Self {
        Self::new(
            sender,
            PacketType::BlockRangeRequest {
                start_height,
                end_height,
            },
        )
    }

    pub fn chain_length_response(sender: Ipv4Addr, chain_length: u64) -> Self {
        Self::new(sender, PacketType::ChainLengthResponse { chain_length })
    }

    pub fn block_range_response(sender: Ipv4Addr, blocks: Vec<Block>) -> Self {
        Self::new(sender, PacketType::BlockRangeResponse { blocks })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut bytes = Vec::new();

        // 패킷 타입 1byte
        bytes.push(self.packet_type_id());

        // 발신자 IP 4bytes
        bytes.extend_from_slice(&self.sender_ip);

        // 페이로드 ..지맘대로
        match &self.payload {
            PacketType::NewBlock { block } => {
                let block_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(block)?;
                bytes.extend_from_slice(&block_bytes);
            }
            PacketType::ChainLengthRequest => {
                // 페이로드가 없다!
            }
            PacketType::BlockRangeRequest {
                start_height,
                end_height,
            } => {
                bytes.extend_from_slice(&start_height.to_le_bytes());
                bytes.extend_from_slice(&end_height.to_le_bytes());
            }
            PacketType::ChainLengthResponse { chain_length } => {
                bytes.extend_from_slice(&chain_length.to_le_bytes());
            }
            PacketType::BlockRangeResponse { blocks } => {
                // 블록 개수 (4 바이트)
                bytes.extend_from_slice(&(blocks.len() as u32).to_le_bytes());
                // [길이: 4 바이트] [블록: 지맘대로]
                for block in blocks {
                    let block_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(block)?;
                    bytes.extend_from_slice(&(block_bytes.len() as u32).to_le_bytes());
                    bytes.extend_from_slice(&block_bytes);
                }
            }
        }

        Ok(bytes)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < 5 {
            return Err(ProtocolError::PacketTooShort {
                expected: 5,
                actual: data.len(),
            });
        }

        let packet_id = data[0];
        let sender_ip: [u8; 4] = data[1..5].try_into().unwrap();
        let payload_data = &data[5..];

        let payload = match packet_id {
            0x01 => {
                let mut aligned: rkyv::util::AlignedVec = rkyv::util::AlignedVec::new();
                aligned.extend_from_slice(payload_data);
                let block = rkyv::from_bytes::<Block, rkyv::rancor::Error>(&aligned)?;
                PacketType::NewBlock { block }
            }
            0x02 => PacketType::ChainLengthRequest,
            0x03 => {
                if payload_data.len() < 16 {
                    return Err(ProtocolError::PayloadTooShort {
                        packet_type: "BlockRangeRequest",
                        expected: 16,
                        actual: payload_data.len(),
                    });
                }
                let start_height = u64::from_le_bytes(payload_data[0..8].try_into().unwrap());
                let end_height = u64::from_le_bytes(payload_data[8..16].try_into().unwrap());
                PacketType::BlockRangeRequest {
                    start_height,
                    end_height,
                }
            }
            0x04 => {
                if payload_data.len() < 8 {
                    return Err(ProtocolError::PayloadTooShort {
                        packet_type: "ChainLengthResponse",
                        expected: 8,
                        actual: payload_data.len(),
                    });
                }
                let chain_length = u64::from_le_bytes(payload_data[0..8].try_into().unwrap());
                PacketType::ChainLengthResponse { chain_length }
            }
            0x05 => {
                if payload_data.len() < 4 {
                    return Err(ProtocolError::PayloadTooShort {
                        packet_type: "BlockRangeResponse",
                        expected: 4,
                        actual: payload_data.len(),
                    });
                }
                let block_count =
                    u32::from_le_bytes(payload_data[0..4].try_into().unwrap()) as usize;
                let mut blocks = Vec::with_capacity(block_count);
                let mut offset = 4;
                for _ in 0..block_count {
                    if payload_data.len() < offset + 4 {
                        return Err(ProtocolError::PayloadTooShort {
                            packet_type: "BlockRangeResponse block length",
                            expected: offset + 4,
                            actual: payload_data.len(),
                        });
                    }
                    let block_len =
                        u32::from_le_bytes(payload_data[offset..offset + 4].try_into().unwrap())
                            as usize;
                    offset += 4;
                    if payload_data.len() < offset + block_len {
                        return Err(ProtocolError::PayloadTooShort {
                            packet_type: "BlockRangeResponse block data",
                            expected: offset + block_len,
                            actual: payload_data.len(),
                        });
                    }
                    let mut aligned: rkyv::util::AlignedVec = rkyv::util::AlignedVec::new();
                    aligned.extend_from_slice(&payload_data[offset..offset + block_len]);
                    let block = rkyv::from_bytes::<Block, rkyv::rancor::Error>(&aligned)?;
                    blocks.push(block);
                    offset += block_len;
                }
                PacketType::BlockRangeResponse { blocks }
            }
            _ => return Err(ProtocolError::UnknownPacketId(packet_id)),
        };

        Ok(ProtocolPacket { sender_ip, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::block::{Block, BlockHeader};
    use crate::network::UdpBroadcast;
    use std::net::Ipv4Addr;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_packet_broadcast() {
        let chain1 = Arc::new(Mutex::new(Vec::<Block>::new()));
        let chain2 = Arc::new(Mutex::new(Vec::<Block>::new()));
        let node1 = UdpBroadcast::with_port(1201, 1201, chain1).unwrap();
        let node2 = UdpBroadcast::with_port(1201, 1201, chain2).unwrap();

        // 보낼 패킷
        let packet = ProtocolPacket::new(Ipv4Addr::new(192, 168, 0, 100), PacketType::ChainLengthRequest);

        let serialized = packet.to_bytes().unwrap();

        let mut receive_buffer = [0u8; 1024]; // 1KB 버퍼 재사용

        let receiver = thread::scope(|scope| {
            let receiver = scope.spawn(|| {
                thread::sleep(Duration::from_millis(100));
                let (size, addr) = node2.recv(&mut receive_buffer).unwrap();
                (size, addr)
            });

            let sender = scope.spawn(|| {
                thread::sleep(Duration::from_millis(50));
                node1.send(&serialized).unwrap();
            });

            sender.join().unwrap();
            receiver.join().unwrap()
        });

        let (received_size, _sender_addr) = receiver;

        let received = &receive_buffer[..received_size];

        let deserialized = ProtocolPacket::from_bytes(received).unwrap();

        // 역직렬화
        assert_eq!(deserialized, packet);
    }

    #[test]
    fn test_size() {
        let packet1 = ProtocolPacket {
            sender_ip: [0, 0, 0, 0],
            payload: PacketType::ChainLengthRequest,
        };

        let packet2 = ProtocolPacket {
            sender_ip: [0, 0, 0, 0],
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
            difficulty: 1.0,
            timestamp: 1000,
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
        let packet = ProtocolPacket::block_range_request(Ipv4Addr::new(172, 16, 0, 1), 10, 20);

        let bytes = packet.to_bytes().unwrap();
        let deserialized = ProtocolPacket::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.packet_type_id(), 0x03);

        match deserialized.payload {
            PacketType::BlockRangeRequest {
                start_height,
                end_height,
            } => {
                assert_eq!(start_height, 10);
                assert_eq!(end_height, 20);
            }
            _ => panic!("Expected BlockRangeRequest"),
        }
    }

    #[test]
    fn test_chain_length_response() {
        let packet = ProtocolPacket::chain_length_response(Ipv4Addr::new(10, 0, 0, 1), 42);

        let bytes = packet.to_bytes().unwrap();
        let deserialized = ProtocolPacket::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.packet_type_id(), 0x04);
        assert!(deserialized.is_response());
        assert!(!deserialized.is_broadcast());

        match deserialized.payload {
            PacketType::ChainLengthResponse { chain_length } => {
                assert_eq!(chain_length, 42);
            }
            _ => panic!("Expected ChainLengthResponse"),
        }
    }

    #[test]
    fn test_block_range_response() {
        let block1 = Block {
            block_header: BlockHeader {
                prev_hash: [0u8; 32],
                height: 1,
                nonce: 100,
                merkle_root: [1u8; 32],
                difficulty: 1.0,
                timestamp: 1000,
            },
            txs: vec![],
        };
        let block2 = Block {
            block_header: BlockHeader {
                prev_hash: [2u8; 32],
                height: 2,
                nonce: 200,
                merkle_root: [3u8; 32],
                difficulty: 2.0,
                timestamp: 2000,
            },
            txs: vec![],
        };

        let packet = ProtocolPacket::block_range_response(
            Ipv4Addr::new(172, 16, 0, 1),
            vec![block1, block2],
        );

        let bytes = packet.to_bytes().unwrap();
        let deserialized = ProtocolPacket::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.packet_type_id(), 0x05);
        assert!(deserialized.is_response());

        match deserialized.payload {
            PacketType::BlockRangeResponse { blocks } => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(blocks[0].block_header.height, 1);
                assert_eq!(blocks[1].block_header.height, 2);
            }
            _ => panic!("Expected BlockRangeResponse"),
        }
    }

    #[test]
    fn test_byte_layout() {
        let ip = Ipv4Addr::new(192, 168, 1, 100);

            // 패킷 및 예상ID
        let packets: Vec<(ProtocolPacket, u8)> = vec![
            (
                ProtocolPacket::new_block(
                    ip,
                    Block {
                        block_header: BlockHeader {
                            prev_hash: [0u8; 32],
                            height: 0,
                            nonce: 0,
                            merkle_root: [0u8; 32],
                            difficulty: 1.0,
                            timestamp: 0,
                        },
                        txs: vec![],
                    },
                ),
                0x01,
            ),
            (ProtocolPacket::chain_length_request(ip), 0x02),
            (ProtocolPacket::block_range_request(ip, 0, 10), 0x03),
            (ProtocolPacket::chain_length_response(ip, 100), 0x04),
            (ProtocolPacket::block_range_response(ip, vec![]), 0x05),
        ];

        for (packet, expected_id) in &packets {
            let bytes = packet.to_bytes().unwrap();

            assert_eq!(bytes[0], *expected_id);

            assert_eq!(&bytes[1..5], &[192, 168, 1, 100]);
        }
    }

    #[test]
    fn test_broadcast_packet_size_ordering() {
        use crate::block::transaction::Transaction;

        let ip = Ipv4Addr::new(192, 168, 1, 1);

        let chain_len_req = ProtocolPacket::chain_length_request(ip);
        let chain_len_req_bytes = chain_len_req.to_bytes().unwrap();
        assert_eq!(ProtocolPacket::from_bytes(chain_len_req_bytes.as_slice()).unwrap(), chain_len_req);

        let block_range_req = ProtocolPacket::block_range_request(ip, 0, 100);
        let block_range_req_bytes = block_range_req.to_bytes().unwrap();
        assert_eq!(ProtocolPacket::from_bytes(block_range_req_bytes.as_slice()).unwrap(), block_range_req);

        let empty_block = Block {
            block_header: BlockHeader {
                prev_hash: [0u8; 32],
                height: 1,
                nonce: 42,
                merkle_root: [0u8; 32],
                difficulty: 1.0,
                timestamp: 1000,
            },
            txs: vec![],
        };
        let new_block_empty = ProtocolPacket::new_block(ip, empty_block);
        let new_block_empty_bytes = new_block_empty.to_bytes().unwrap();
        assert_eq!(ProtocolPacket::from_bytes(new_block_empty_bytes.as_slice()).unwrap(), new_block_empty);

        let block_with_tx = Block {
            block_header: BlockHeader {
                prev_hash: [0u8; 32],
                height: 2,
                nonce: 99,
                merkle_root: [0u8; 32],
                difficulty: 1.0,
                timestamp: 2000,
            },
            txs: vec![Transaction::new([1u8; 32], [2u8; 32], 100, [3u8; 32])],
        };
        let new_block_with_tx = ProtocolPacket::new_block(ip, block_with_tx);
        let new_block_with_tx_bytes = new_block_with_tx.to_bytes().unwrap();
        assert_eq!(ProtocolPacket::from_bytes(new_block_with_tx_bytes.as_slice()).unwrap(), new_block_with_tx);

        assert_eq!(chain_len_req_bytes.len(), 5, "ChainLengthRequest: 5 bytes (1 type + 4 IP)");
        assert_eq!(block_range_req_bytes.len(), 21, "BlockRangeRequest: 21 bytes (5 header + 16 data)");

        assert!(
            chain_len_req_bytes.len() < block_range_req_bytes.len(),
            "ChainLengthRequest ({} B) < BlockRangeRequest ({} B)",
            chain_len_req_bytes.len(),
            block_range_req_bytes.len(),
        );
        assert!(
            block_range_req_bytes.len() < new_block_empty_bytes.len(),
            "BlockRangeRequest ({} B) < NewBlock empty ({} B)",
            block_range_req_bytes.len(),
            new_block_empty_bytes.len(),
        );
        assert!(
            new_block_empty_bytes.len() < new_block_with_tx_bytes.len(),
            "NewBlock empty ({} B) < NewBlock with tx ({} B)",
            new_block_empty_bytes.len(),
            new_block_with_tx_bytes.len(),
        );
    }

    #[test]
    fn test_all_sizes_differ() {
        let ip = Ipv4Addr::new(0, 0, 0, 0);

        let packets = vec![
            ProtocolPacket::new_block(
                ip,
                Block {
                    block_header: BlockHeader {
                        prev_hash: [0u8; 32],
                        height: 0,
                        nonce: 0,
                        merkle_root: [0u8; 32],
                        difficulty: 1.0,
                        timestamp: 0,
                    },
                    txs: vec![],
                },
            ),
            ProtocolPacket::chain_length_request(ip),
            ProtocolPacket::block_range_request(ip, 0, 0),
            ProtocolPacket::chain_length_response(ip, 0),
            ProtocolPacket::block_range_response(ip, vec![]),
        ];

        let sizes: Vec<usize> = packets
            .iter()
            .map(|p| p.to_bytes().unwrap().len())
            .collect();

        for i in 0..sizes.len() {
            for j in (i + 1)..sizes.len() {
                assert_ne!(
                    sizes[i], sizes[j],
                    "packet {} and {} have the same byte length: {}",
                    i, j, sizes[i]
                );
            }
        }
    }
}
