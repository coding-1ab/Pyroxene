use socket2::{Domain, Protocol, Socket, Type};
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use rkyv::from_bytes;
use rkyv::rancor::Error as RkyvError;
use crate::block::block::Block;

mod protocol;

pub const PORT: u16 = 1200;

pub struct UdpBroadcast {
    socket: UdpSocket,
    target: SocketAddr,
    chain: Arc<Mutex<Vec<Block>>>,
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
            .get(block.block_header.id as usize)
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

        if self.is_known(&block) { // 이미본거야 콘
            return Ok(());
        }

        self.mark_known(block);
        self.send(received_data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use crate::network::UdpBroadcast;

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