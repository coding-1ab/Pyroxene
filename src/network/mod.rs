use crate::network::protocol::{Packet, WalletId};
use rkyv::rancor::Error;
use rkyv::{from_bytes, to_bytes};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};

mod protocol;

pub const PORT: u16 = 1200;

pub struct UdpBroadcast {
    id: WalletId,
    buffer: Box<[u8; 1024]>,
    socket: UdpSocket,
    target: SocketAddr,
}

impl UdpBroadcast {
    pub fn new(id: WalletId) -> Result<Self> {
        Self::with_port(id, PORT, PORT)
    }

    pub fn with_port(id: WalletId, send: u16, receive: u16) -> Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_broadcast(true)?;
        socket.set_reuse_address(true)?;

        let addr: SocketAddr = ([0, 0, 0, 0], receive).into(); // 0.0.0.0 != 192.168.x.x.. 미래의 나, 해결해라!
        socket.bind(&addr.into())?;

        let socket: UdpSocket = socket.into();

        Ok(Self {
            id,
            buffer: unsafe { Box::new_zeroed().assume_init() },
            socket,
            target: ([255, 255, 255, 255], send).into(), // 브로드캐스트 주소 바꿀 것
        })
    }

    pub fn send(&self, data: &Packet) -> Result<usize> {
        self.socket.send_to(to_bytes::<Error>(data).unwrap().as_slice(), self.target)
    }

    pub fn recv(&mut self) -> Result<Packet> {
        loop {
            let (length, _) = self.socket.recv_from(self.buffer.as_mut_slice())?;
            if length == self.buffer.len() {
                panic!("Buffer Overflow");
            }

            let contents = &self.buffer.as_slice()[..length];
            let packet = from_bytes::<Packet, Error>(contents).unwrap();
            if packet.sender_id == self.id {
                continue;
            }

            break Ok(packet)
        }
    }

    pub fn is_self(&self, addr: SocketAddr) -> bool {
        addr == self.target
    }
}
