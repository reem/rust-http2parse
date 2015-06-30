use {Payload, Error, Flag, Kind, StreamIdentifier, ParserSettings, FRAME_HEADER_BYTES};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Frame<'a> {
    pub header: FrameHeader,
    pub payload: Payload<'a>
}

impl<'a> Frame<'a> {
    pub fn parse(header: FrameHeader, buf: &[u8],
                 settings: ParserSettings) -> Result<Frame, Error> {
        Ok(Frame {
            header: header,
            payload: try!(Payload::parse(header, buf, settings))
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
            kind: Kind::new(buf[3]),
            flag: try!(Flag::new(buf[4]).map_err(|()| { Error::BadFlag(buf[4]) })),
            id: StreamIdentifier::parse(&buf[5..])
        })
    }
}

