use {Payload, Error, Flag, Kind, StreamIdentifier, ParserSettings, FRAME_HEADER_BYTES};

#[derive(Copy, Clone, Debug)]
pub struct Frame<'a> {
    header: FrameHeader,
    payload: Payload<'a>
}

impl<'a> Frame<'a> {
    pub fn parse(buf: &[u8], settings: ParserSettings) -> Result<Frame, Error> {
        if buf.len() < FRAME_HEADER_BYTES {
            let header = FrameHeader {
                length: ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | buf[2] as u32,
                kind: try!(Kind::new(buf[3]).map_err(|()| { Error::BadKind(buf[3]) })),
                flag: try!(Flag::new(buf[4]).map_err(|()| { Error::BadFlag(buf[4]) })),
                id: StreamIdentifier::parse(&buf[5..])
            };

            header.parse_payload(&buf[FRAME_HEADER_BYTES..], settings)
        } else {
            Err(Error::Short)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FrameHeader {
    length: u32,
    kind: Kind,
    flag: Flag,
    id: StreamIdentifier,
}

impl FrameHeader {
    fn parse_payload(self, buf: &[u8], settings: ParserSettings) -> Result<Frame, Error> {
        if buf.len() >= self.length as usize {
            return Err(Error::Short)
        }

        Ok(Frame {
            header: self,
            payload: match self.kind {
                Kind::Data => {
                    try!(parse_payload_with_padding(settings, self, buf, |buf| {
                        Payload::Data {
                            data: buf
                        }
                    }))
                },
                _ => panic!("unimplemented")
            }
        })
    }
}

fn parse_payload_with_padding<'a, F>(settings: ParserSettings, header: FrameHeader,
                                 buf: &'a [u8], cb: F) -> Result<Payload, Error>
where F: FnOnce(&'a [u8]) -> Payload {
    if settings.padding {
        let pad_length = buf[0];
        if pad_length as u32 > header.length {
            Err(Error::TooMuchPadding(pad_length))
        } else {
            Ok(cb(&buf[1..header.length as usize - pad_length as usize]))
        }
    } else {
        Ok(cb(buf))
    }
}

