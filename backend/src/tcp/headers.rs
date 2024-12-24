//IP and TCP header structures

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
