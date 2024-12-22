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
}
