//Calculation utilities for TCP checksums

pub fn calculate_ip_checksum(header: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    for chunk in header.chunks(2) {
        let word = if chunk.len() == 2 {
            ((chunk[0] as u32) << 8) | chunk[1] as u32 //<<8 is to push them 8 bits to the side so we dont have problems. ex : 0x01 + 0x02 should not be 0x03.
        } else {
            (chunk[0] as u32) << 8
        };
        sum = sum.wrapping_add(word);
    } //wouldnt need to do this if we were adding the binaries since theyd already be aligned

    // Add carried bits
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !sum as u16
}

pub fn calculate_tcp_checksum(
    tcp_header: &[u8],
    data: &[u8],
    src_addr: [u8; 4],
    dst_addr: [u8; 4],
) -> u16 {
    let mut sum: u32 = 0;

    

    !sum as u16
}
