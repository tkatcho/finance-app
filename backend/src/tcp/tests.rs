#[cfg(test)]
mod tests {
    use crate::tcp::{IpHeader, TcpHeader};


    #[test]
    fn test_ip_header_creation() {
        let src_addr = [192, 168, 1, 1];
        let dst_addr = [192, 168, 1, 2];
        let header = IpHeader::new(src_addr, dst_addr, 20); // 20 bytes for header

        assert_eq!(header.version_ihl, 0x45); // IPv4 + 5 words
        assert_eq!(header.time_to_live, 64);
        assert_eq!(header.protocol, 6); // TCP
        assert_eq!(header.src_addr, src_addr);
        assert_eq!(header.dst_addr, dst_addr);
        assert_eq!(header.ip_flags, 0x4000); // Don't fragment
    }

    #[test]
    fn test_tcp_header_flags() {
        // Test SYN flag
        let syn_header = TcpHeader::new(
            1234,  // src_port
            80,    // dst_port
            100,   // seq_num
            0,     // ack_num
            true,  // syn
            false, // ack
            false, // fin
        );

        let flags = syn_header.get_flags();
        assert!(flags.syn);
        assert!(!flags.ack);
        assert!(!flags.fin);

        // Test SYN-ACK flags
        let syn_ack_header = TcpHeader::new(
            80,    // src_port
            1234,  // dst_port
            200,   // seq_num
            101,   // ack_num
            true,  // syn
            true,  // ack
            false, // fin
        );

        let flags = syn_ack_header.get_flags();
        assert!(flags.syn);
        assert!(flags.ack);
        assert!(!flags.fin);

        // Test FIN-ACK flags
        let fin_ack_header = TcpHeader::new(
            1234,  // src_port
            80,    // dst_port
            300,   // seq_num
            201,   // ack_num
            false, // syn
            true,  // ack
            true,  // fin
        );

        let flags = fin_ack_header.get_flags();
        assert!(!flags.syn);
        assert!(flags.ack);
        assert!(flags.fin);
    }

    #[test]
    fn test_header_size() {
        // Ensure headers are the correct size
        assert_eq!(std::mem::size_of::<IpHeader>(), 20);
        assert_eq!(std::mem::size_of::<TcpHeader>(), 20);
    }

    #[test]
    fn test_ip_fragmentation() {
        let mut header = IpHeader::new([127, 0, 0, 1], [127, 0, 0, 1], 20);

        // Test don't fragment flag
        assert_eq!(header.ip_flags & 0x4000, 0x4000);

        // Test fragment offset
        header.ip_flags = 0x2000; // More fragments
        assert_eq!(header.ip_flags & 0x2000, 0x2000);
    }

    #[test]
    fn test_tcp_sequence_numbers() {
        let header = TcpHeader::new(
            1234,       // src_port
            80,         // dst_port
            0xFFFFFFFF, // seq_num (max value)
            0,          // ack_num
            false,      // syn
            false,      // ack
            false,      // fin
        );

        assert_eq!(header.seq_num, 0xFFFFFFFF);

        // Test sequence number wraparound
        let next_seq = header.seq_num.wrapping_add(1);
        assert_eq!(next_seq, 0);
    }

    #[test]
    fn test_header_to_bytes() {
        let ip_header = IpHeader::new([192, 168, 1, 1], [192, 168, 1, 2], 20);
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &ip_header as *const IpHeader as *const u8,
                std::mem::size_of::<IpHeader>(),
            )
        };

        println!("{:?}", bytes);

        assert_eq!(bytes.len(), 20);
        assert_eq!(bytes[0], 0x45); // version_ihl
        assert_eq!(bytes[8], 64); // ttl
    }
}
