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

#[cfg(test)]
mod test {
    use {Kind, Flag, FrameHeader, StreamIdentifier};

    #[test]
    fn test_frame_header_parse_empty() {
        assert_eq!(FrameHeader {
            length: 0,
            kind: Kind::Data,
            flag: Flag::empty(),
            id: StreamIdentifier(0)
        }, FrameHeader::parse(&[
            0u8, 0u8, 0u8, // length
            0u8, // type/kind
            0u8, // flags
            0u8, 0u8, 0u8, 0u8 // reserved bit + stream identifier
        ]).unwrap());
    }

    #[test]
    fn test_frame_header_parse_full() {
        assert_eq!(FrameHeader {
            length: 16777215,
            kind: Kind::Unregistered,
            flag: Flag::empty(),
            id: StreamIdentifier(2147483647)
        }, FrameHeader::parse(&[
            0xFF, 0xFF, 0xFF, // length
            0xFF, // type/kind
            0x0, // flags
            0xFF, 0xFF, 0xFF, 0xFF // reserved bit + stream identifier
        ]).unwrap());
    }

    #[test]
    fn test_frame_header_parse() {
        assert_eq!(FrameHeader {
            length: 66051,
            kind: Kind::Settings,
            flag: Flag::end_stream(),
            id: StreamIdentifier(101124105)
        }, FrameHeader::parse(&[
            0x1, 0x2, 0x3, // length
            0x4, // type/kind
            0x1, // flags
            0x6, 0x7, 0x8, 0x9 // reserved bit + stream identifier
        ]).unwrap());
    }
}

