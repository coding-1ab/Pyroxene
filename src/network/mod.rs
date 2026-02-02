use crate::block::block::Block;
use rkyv::rancor::Error as RkyvError;
use rkyv::from_bytes;
use socket2::{Domain, Protocol, Socket, Type};
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub mod protocol;

pub const PORT: u16 = 1200;

pub struct UdpBroadcast {
    socket: UdpSocket,
    target: SocketAddr,
    pub chain: Arc<Mutex<Vec<Block>>>,
}

impl UdpBroadcast {
    pub fn new() -> Result<Self> {
        Self::with_port(PORT, PORT)
    }

    pub fn with_port(send: u16, receive: u16) -> Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_broadcast(true)?;
        socket.set_reuse_address(true)?;

        let addr: SocketAddr = ([0, 0, 0, 0], receive).into(); // 0.0.0.0 != 192.168.x.x.. 미래의 나, 해결해라!
        socket.bind(&addr.into())?;

        let socket: UdpSocket = socket.into();

        Ok(Self {
            socket,
            target: ([255, 255, 255, 255], send).into(), // 브로드캐스트 주소 바꿀 것
            chain: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn spawn(self, receive_block: Receiver<Block>, notify_block: Sender<()>) {
        thread::spawn(move || {
            thread::scope(|scope| {
                scope.spawn(|| {
                    let receive_block = receive_block;
                    let chain = self.chain.clone();
                    loop {
                        let new_block = receive_block.recv().unwrap();

                        let sender_ip = [0, 0, 0, 0];
                        let packet = protocol::ProtocolPacket {
                            sender_ip,
                            payload: protocol::PacketType::NewBlock { block: new_block.clone() },
                        };

                        let bytes = packet.to_bytes().unwrap();
                        let mut chain_access = chain.lock().unwrap();
                        chain_access.push(new_block);
                        println!("Sent new block");
                        println!("Chain: {}", chain_access.len());
                        drop(chain_access);
                        self.send(bytes.as_slice()).unwrap();
                    }
                });

                scope.spawn(|| {
                    let notify_block = notify_block;
                    let mut buffer = Box::new([0u8; 4096]);
                    let chain = self.chain.clone();
                    loop {
                        let (length, source) = self.recv(buffer.as_mut_slice()).unwrap();
                        let data = &buffer.as_slice()[..length];

                        let new_block = if let Ok(packet) = protocol::ProtocolPacket::from_bytes(data) {
                            match packet.payload {
                                protocol::PacketType::NewBlock { block } => Some(block),
                                _ => {
                                    println!("Received non-NewBlock packet type: 0x{:02x}, ignoring",
                                        packet.packet_type_id());
                                    None
                                }
                            }
                        } else {
                            from_bytes::<Block, RkyvError>(data).ok()
                        };

                        if let Some(new_block) = new_block {
                            let mut chain_access = chain.lock().unwrap();
                            if chain_access.len() == new_block.block_header.height as usize {
                                chain_access.push(new_block);
                                println!("Received new block from source: {}", source);
                                println!("Chain: {}", chain_access.len());

                                notify_block.send(()).unwrap();
                            }
                            drop(chain_access);
                        } else {
                            eprintln!("Failed to deserialize packet from {}", source);
                        }
                    }
                });
            })
        });
    }

    pub fn send(&self, data: &[u8]) -> Result<usize> {
        self.socket.send_to(data, self.target)
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    pub fn is_self(&self, addr: SocketAddr) -> bool {
        addr == self.target
    }

    pub fn is_known(&self, block: &Block) -> bool {
        self.chain
            .lock()
            .unwrap()
            .get(block.block_header.height as usize)
            .is_some()
    }

    pub fn mark_known(&self, block: Block) {
        self.chain.lock().unwrap().push(block);
    }

    pub fn receive_and_rebroadcast(&self) -> Result<()> {
        let mut buffer = vec![0u8; 65536];
        let (size, _addr) = self.recv(&mut buffer)?;
        let received_data = &buffer[..size];
        let block: Block = from_bytes::<Block, RkyvError>(received_data).unwrap();

        if self.is_known(&block) {
            // 이미본거야 콘
            return Ok(());
        }

        self.mark_known(block);
        self.send(received_data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::network::UdpBroadcast;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_broadcast() {
        let node1 = UdpBroadcast::new().unwrap();
        let node2 = UdpBroadcast::new().unwrap();
        let data: [u8; 4] = [0x43, 0x55, 0x54, 0x45];
        let mut receive_buffer = [0u8; 4];

        let receiver = thread::scope(|scope| {
            let result = scope.spawn(|| {
                println!("Receiving..");
                let result = node2.recv(&mut receive_buffer).unwrap();
                println!("Received");
                result
            });
            scope.spawn(|| {
                thread::sleep(Duration::from_secs(1));
                println!("Sending");
                node1.send(&data).unwrap();
            });
            result.join().unwrap()
        });
        let (receive_count, sender_address) = receiver;

        assert_eq!(receive_count, data.len());
        assert_eq!(receive_buffer, data);
    }

    const DATA: [u8; 4] = [1, 2, 3, 4];

    #[test]
    fn send() {
        let node = UdpBroadcast::new().unwrap();
        node.send(&DATA).unwrap();
    }

    #[test]
    fn receive() {
        let node = UdpBroadcast::new().unwrap();
        let mut buffer = [0u8; 64];
        let (length, sender) = node.recv(&mut buffer).unwrap();
        let received = &buffer[..length];

        println!("Received: {:?} from {}", received, sender);
    }
}
