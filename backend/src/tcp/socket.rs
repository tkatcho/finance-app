use std::mem::MaybeUninit;
use std::{io, net::Ipv4Addr};

use socket2::{Domain, Protocol, Socket, Type};

use super::{
    checksum::calculate_tcp_checksum, IpHeader, TcpHeader, IP_HEADER_SIZE, TCP_HEADER_SIZE,
};
#[derive(Debug)]
pub struct RawSocket {
    socket: Socket,
    pub local_addr: [u8; 4],
    pub local_port: u16,
    seq_num: u32,
}

impl RawSocket {
    pub fn new() -> io::Result<Self> {
        // Create TCP stream socket
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

        socket.set_nonblocking(true)?;
        socket.set_reuse_address(true)?;

        Ok(Self {
            socket,
            local_addr: [0; 4],
            local_port: 0,
            seq_num: rand::random(),
        })
    }
    pub fn new_with_params(
        socket: Socket,
        local_addr: [u8; 4],
        local_port: u16,
    ) -> io::Result<Self> {
        // Create TCP stream socket
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

        socket.set_nonblocking(true)?;
        socket.set_reuse_address(true)?;

        Ok(Self {
            socket,
            local_addr,
            local_port,
            seq_num: rand::random(),
        })
    }
    pub fn bind(&mut self, addr: &str, port: u16) -> io::Result<()> {
        let ip: Ipv4Addr = addr
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let addr =
            socket2::SockAddr::from(std::net::SocketAddr::new(std::net::IpAddr::V4(ip), port));

        println!("Binding to {:?}:{}", addr, port);
        self.socket.bind(&addr)?;
        self.local_addr = ip.octets();
        self.local_port = port;

        println!("Starting to listen...");
        self.socket.listen(128)?;
        println!("Listen successful");

        Ok(())
    }
    pub fn accept(&self) -> io::Result<(Socket, std::net::SocketAddr)> {
        self.socket
            .accept()
            .map(|(socket, addr)| (socket, addr.as_socket().unwrap()))
    }

    pub fn send_packet(
        &mut self,
        dst_addr: [u8; 4],
        dst_port: u16,
        syn: bool,
        ack: bool,
        fin: bool,
        data: &[u8],
    ) -> io::Result<usize> {
        let mut packet = Vec::with_capacity(IP_HEADER_SIZE + TCP_HEADER_SIZE + data.len());

        let ip_header = IpHeader::new(self.local_addr, dst_addr, packet.len() as u16);

        let mut tcp_header =
            TcpHeader::new(self.local_port, dst_port, self.seq_num, 0, syn, ack, fin);

        let tcp_bytes = tcp_header.to_bytes();

        let checksum = calculate_tcp_checksum(&tcp_bytes, data, self.local_addr, dst_addr);

        tcp_header.checksum = checksum;

        packet.extend_from_slice(&ip_header.to_bytes());
        packet.extend_from_slice(&tcp_bytes);
        packet.extend_from_slice(data);

        let addr = socket2::SockAddr::from(std::net::SocketAddrV4::new(dst_addr.into(), dst_port));
        self.socket.send_to(&packet, &addr)
    }

    pub fn receive_packet(&self) -> io::Result<(Vec<u8>, [u8; 4], u16)> {
        // Define the buffer with uninitialized memory (max IP packet size: 65535 bytes)
        let mut buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); 65535];

        // Receive a packet into the buffer
        let (size, _) = self.socket.recv_from(&mut buffer)?;

        // Ensure the packet is large enough for IP and TCP headers
        if size < IP_HEADER_SIZE + TCP_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Packet too small: insufficient header size",
            ));
        }

        // Safely convert the buffer into a slice of initialized bytes
        let buffer = unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, size) };

        // Extract IP header
        let ip_header = IpHeader::from_bytes(&buffer[..IP_HEADER_SIZE])?;
        let src_addr = ip_header.src_addr;

        // Extract TCP header
        let tcp_header =
            TcpHeader::from_bytes(&buffer[IP_HEADER_SIZE..IP_HEADER_SIZE + TCP_HEADER_SIZE])?;
        let src_port = tcp_header.src_port;

        // Extract payload data
        let data = &buffer[IP_HEADER_SIZE + TCP_HEADER_SIZE..];

        // Validate TCP checksum
        let checksum = calculate_tcp_checksum(
            &buffer[IP_HEADER_SIZE..IP_HEADER_SIZE + TCP_HEADER_SIZE],
            data,
            src_addr,
            ip_header.dst_addr,
        );
        if checksum != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TCP checksum mismatch: packet corrupted",
            ));
        }

        // Return the payload, source address, and source port
        Ok((data.to_vec(), src_addr, src_port))
    }

    pub fn set_buffer_size(&self, size: usize) -> io::Result<()> {
        self.socket.set_recv_buffer_size(size)?;
        self.socket.set_send_buffer_size(size)?;
        Ok(())
    }

    pub fn set_timeout(&self, seconds: u64) -> io::Result<()> {
        let timeout = std::time::Duration::from_secs(seconds);
        self.socket.set_read_timeout(Some(timeout))?;
        self.socket.set_write_timeout(Some(timeout))?;
        Ok(())
    }

    pub fn get_sequence_num(&self) -> u32 {
        self.seq_num
    }

    pub fn getSocket(&self) -> &Socket {
        &self.socket
    }
    pub fn increment_sequence_num(&mut self, increment: u32) {
        self.seq_num = self.seq_num.wrapping_add(increment);
    }
}
