#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Flag {
    // 0x1 is ACK on SETTINGS frames but END_STREAM on other frames.
    EndStreamOrAck = 0x1,
    EndHeaders = 0x4,
    Padded = 0x8,
    Priority = 0x20
}

impl Flag {
    pub fn new(byte: u8) -> Result<Flag, ()> {
        match byte {
            0x1 => Ok(Flag::EndStreamOrAck),
            0x4 => Ok(Flag::EndHeaders),
            0x8 => Ok(Flag::Padded),
            0x20 => Ok(Flag::Priority),
            _ => Err(())
        }
    }
}

