//IP and TCP header structures

use std::io;

pub struct IpHeader {
    pub version_ihl: u8,
    pub tos: u8,
    pub total_length: u16,
    pub identification: u16,
    pub ip_flags: u16,
    pub time_to_live: u8,
    pub protocol: u8,
    pub header_checksum: u16,
    pub src_addr: [u8; 4],
    pub dst_addr: [u8; 4],
}

pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub data_offset_flags: u16,
    pub window: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
}

pub struct TcpFlags {
    pub syn: bool,
    pub fin: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
}

impl IpHeader {
    pub fn new(src_addr: [u8; 4], dst_addr: [u8; 4], total_length: u16) -> Self {
        Self {
            version_ihl: 0x45,
            tos: 0,
            total_length,
            identification: rand::random(),
            ip_flags: 0x4000,
            time_to_live: 64,
            protocol: 6, //tcp
            header_checksum: 0,
            src_addr,
            dst_addr,
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        // Check minimum length
        if bytes.len() < 20 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "IP header too short",
            ));
        }

        // Extract IP version and IHL
        let version = (bytes[0] >> 4) & 0xF;
        if version != 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not an IPv4 packet",
            ));
        }

        let header = IpHeader {
            version_ihl: bytes[0],
            tos: bytes[1],
            total_length: u16::from_be_bytes([bytes[2], bytes[3]]),
            identification: u16::from_be_bytes([bytes[4], bytes[5]]),
            ip_flags: u16::from_be_bytes([bytes[6], bytes[7]]),
            time_to_live: bytes[8],
            protocol: bytes[9],
            header_checksum: u16::from_be_bytes([bytes[10], bytes[11]]),
            src_addr: [bytes[12], bytes[13], bytes[14], bytes[15]],
            dst_addr: [bytes[16], bytes[17], bytes[18], bytes[19]],
        };

        // Validate header length
        let ihl = (header.version_ihl & 0xF) * 4;
        if ihl < 20 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid IP header length",
            ));
        }

        // Validate total length
        if (header.total_length as usize) < (ihl as usize) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Total length smaller than header length",
            ));
        }

        Ok(header)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20); // Minimum IP header size is 20 bytes

        bytes.push(self.version_ihl);
        bytes.push(self.tos);
        bytes.extend_from_slice(&self.total_length.to_be_bytes());
        bytes.extend_from_slice(&self.identification.to_be_bytes());
        bytes.extend_from_slice(&self.ip_flags.to_be_bytes());
        bytes.push(self.time_to_live);
        bytes.push(self.protocol);
        bytes.extend_from_slice(&self.header_checksum.to_be_bytes());
        bytes.extend_from_slice(&self.src_addr);
        bytes.extend_from_slice(&self.dst_addr);

        bytes
    }
}

impl TcpHeader {
    pub fn new(
        src_port: u16,
        dst_port: u16,
        seq_num: u32,
        ack_num: u32,
        syn: bool,
        ack: bool,
        fin: bool,
    ) -> Self {
        let mut flags = 0x5000;
        if syn {
            flags |= 0x0002;
        }
        if ack {
            flags |= 0x0010;
        }
        if fin {
            flags |= 0x0001;
        }
        Self {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            data_offset_flags: flags,
            window: 65535,
            checksum: 0,
            urgent_pointer: 0,
        }
    }
    pub fn get_flags(&self) -> TcpFlags {
        TcpFlags::from_raw(self.data_offset_flags & 0x3F)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20); // Minimum TCP header size is 20 bytes

        bytes.extend_from_slice(&self.src_port.to_be_bytes());
        bytes.extend_from_slice(&self.dst_port.to_be_bytes());
        bytes.extend_from_slice(&self.seq_num.to_be_bytes());
        bytes.extend_from_slice(&self.ack_num.to_be_bytes());
        bytes.push(((self.data_offset_flags << 4) | (self.get_flags().to_raw() >> 4)) as u8);
        bytes.push((self.get_flags().to_raw() & 0x0F) as u8);
        bytes.extend_from_slice(&self.window.to_be_bytes());
        bytes.extend_from_slice(&self.checksum.to_be_bytes());
        bytes.extend_from_slice(&self.urgent_pointer.to_be_bytes());

        bytes
    }
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        // Check minimum length
        if bytes.len() < 20 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TCP header too short",
            ));
        }

        let header = TcpHeader {
            src_port: u16::from_be_bytes([bytes[0], bytes[1]]),
            dst_port: u16::from_be_bytes([bytes[2], bytes[3]]),
            seq_num: u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            ack_num: u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            data_offset_flags: u16::from_be_bytes([bytes[12], bytes[13]]),
            window: u16::from_be_bytes([bytes[14], bytes[15]]),
            checksum: u16::from_be_bytes([bytes[16], bytes[17]]),
            urgent_pointer: u16::from_be_bytes([bytes[18], bytes[19]]),
        };

        // Validate data offset
        let data_offset = (header.data_offset_flags >> 12) & 0xF;
        if data_offset < 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TCP header length too small",
            ));
        }
        Ok(header)
    }
}

impl TcpFlags {
    pub fn from_raw(flags: u16) -> Self {
        //we are inside of the flags so we have acess to those bits
        Self {
            fin: flags & 0x01 != 0,
            syn: flags & 0x02 != 0,
            rst: flags & 0x04 != 0,
            psh: flags & 0x08 != 0,
            ack: flags & 0x10 != 0,
            urg: flags & 0x20 != 0,
        }
    }

    pub fn to_raw(&self) -> u16 {
        let mut flags = 0;
        if self.fin {
            flags |= 0x01;
        }
        if self.syn {
            flags |= 0x02;
        }
        if self.rst {
            flags |= 0x04;
        }
        if self.psh {
            flags |= 0x08;
        }
        if self.ack {
            flags |= 0x10;
        }
        if self.urg {
            flags |= 0x20;
        }
        flags
    }
}
