use {Payload, Error, Flag, Kind, StreamIdentifier, ParserSettings, FRAME_HEADER_BYTES};

#[derive(Copy, Clone, Debug)]
pub struct Frame<'a> {
    pub header: FrameHeader,
    pub payload: Payload<'a>
}

const PRIORITY_BYTES: u32 = 5;
const PADDING_BYTES: u32 = 1;

impl<'a> Frame<'a> {
    pub fn parse(header: FrameHeader, buf: &[u8],
                 settings: ParserSettings) -> Result<Frame, Error> {
        if buf.len() < header.length as usize {
            return Err(Error::Short)
        }

        let min_payload_length =
            if settings.priority && settings.padding {
                PRIORITY_BYTES + PADDING_BYTES
            } else if settings.priority {
                PRIORITY_BYTES
            } else if settings.padding {
                PADDING_BYTES
            } else {
                0
            };

        if header.length < min_payload_length {
            return Err(Error::PayloadLengthTooShort)
        }

        Ok(Frame {
            header: header,
            payload: try!(Payload::parse(header, &buf[..header.length as usize], settings))
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}

impl FrameHeader {
    pub fn parse(buf: &[u8]) -> Result<FrameHeader, Error> {
        if buf.len() > FRAME_HEADER_BYTES {
            return Err(Error::Short);
        }

        Ok(FrameHeader {
            length: ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | buf[2] as u32,
            kind: try!(Kind::new(buf[3]).map_err(|()| { Error::BadKind(buf[3]) })),
            flag: try!(Flag::new(buf[4]).map_err(|()| { Error::BadFlag(buf[4]) })),
            id: StreamIdentifier::parse(&buf[5..])
        })
    }
}

