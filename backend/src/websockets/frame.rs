#[derive(Debug)]
pub struct Frame {
    pub fin: bool,
    pub op_code: OpCode, //tells what king of frame we have: 0x1 text, 0x2 binary, 0x8 connection closed, 0x9 ping, 0xA pong
    pub mask: bool,
    pub payload_len: u64,
    pub mask_key: Option<[u8; 4]>, //array of 4 items of 1 byte each
    pub payload: Vec<u8>,          //array that can keep growing with 1 byte each
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    ConnectionClosed = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl Frame {
    pub fn new(opcode: OpCode, payload: Vec<u8>) -> Self {
        Frame {
            fin: true,
            op_code: opcode,
            mask: false,
            payload_len: payload.len() as u64,
            mask_key: None,
            payload,
        }
    }

    pub fn parse(
        fin: bool,
        opcode: OpCode,
        masked: bool,
        payload_len: u64,
    ) -> Result<Frame, std::io::Error> {
        Ok(Frame {
            fin,
            op_code: OpCode::from(opcode),
            mask: masked,
            payload_len,
            mask_key: None,
            payload: Vec::new(),
        })
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // First byte: FIN bit and opcode
        let mut first_byte = if self.fin { 0x80 } else { 0x00 };
        first_byte |= self.op_code as u8;
        bytes.push(first_byte);

        // Second byte: MASK bit and payload length
        let mut second_byte = if self.mask { 0x80 } else { 0x00 };

        // Handle different payload length cases
        if self.payload_len <= 125 {
            second_byte |= self.payload_len as u8;
            bytes.push(second_byte);
        } else if self.payload_len <= 65535 {
            second_byte |= 126;
            bytes.push(second_byte);
            bytes.extend(&(self.payload_len as u16).to_be_bytes());
        } else {
            second_byte |= 127;
            bytes.push(second_byte);
            bytes.extend(&(self.payload_len as u64).to_be_bytes());
        }

        // Add masking key if present
        if let Some(mask_key) = self.mask_key {
            bytes.extend(&mask_key);
        }

        // Add payload (applying mask if necessary)
        if self.mask {
            if let Some(mask_key) = self.mask_key {
                let masked_payload: Vec<u8> = self
                    .payload
                    .iter()
                    .enumerate()
                    .map(|(i, &byte)| byte ^ mask_key[i % 4])
                    .collect();
                bytes.extend(masked_payload);
            }
        } else {
            bytes.extend(&self.payload);
        }

        bytes
    }
}
