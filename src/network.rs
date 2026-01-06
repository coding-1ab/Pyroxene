use socket2::{Domain, Protocol, Socket, Type};
use std::io::Result;
use std::net::{SocketAddr, UdpSocket};

pub struct UdpBroadcast {
    socket: UdpSocket,
    target: SocketAddr,
    local: SocketAddr,
}

impl UdpBroadcast {
    pub fn new(port: u16) -> Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        socket.set_broadcast(true)?;

        let addr: SocketAddr = ([0, 0, 0, 0], port).into(); // 0.0.0.0 != 192.168.x.x.. 미래의 나, 해결해라!
        socket.bind(&addr.into())?;

        let socket: UdpSocket = socket.into();
        let local = socket.local_addr()?;

        Ok(Self {
            socket,
            target: ([255, 255, 255, 255], port).into(), // 브로드캐스트 주소 바꿀 것
            local,
        })
    }

    pub fn send(&self, data: &[u8]) -> Result<usize> {
        self.socket.send_to(data, self.target)
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local
    }

    pub fn is_self(&self, addr: SocketAddr) -> bool {
        addr == self.local
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    #[test]
    fn test_broadcast() {
        let (sender, listener) = mpsc::channel();
        let data: [u8; 4] = [0x43, 0x55, 0x54, 0x45];
        sender.send(data).unwrap();
        let received = listener.recv().unwrap();
        assert_eq!(received, data);

    }

}