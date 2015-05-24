use {FrameHeader, StreamIdentifier, Error, Kind, ParserSettings};

#[derive(Copy, Clone, Debug)]
pub enum Payload<'a> {
    Data {
        data: &'a [u8]
    },
    Headers {
        priority: Option<Priority>,
        block: &'a [u8]
    },
    Priority(Priority),
    Reset(u32),
    Settings(&'a [Setting]),
    PushPromise {
        id: StreamIdentifier,
        block: &'a [u8]
    },
    Ping(u64),
    GoAway {
        last: StreamIdentifier,
        error: u32
    },
    WindowUpdate {
        size_increment: u32
    }
}

impl<'a> Payload<'a> {
    pub fn parse(header: FrameHeader, buf: &'a [u8],
                 settings: ParserSettings) -> Result<Payload<'a>, Error> {
        match header.kind {
            Kind::Data => Payload::parse_data(header, buf, settings),
            Kind::Headers => Payload::parse_headers(header, buf, settings),
            _ => panic!("unimplemented")
        }
    }

    fn parse_data(header: FrameHeader, buf: &'a [u8],
                  settings: ParserSettings) -> Result<Payload<'a>, Error> {
        Ok(Payload::Data {
            data: try!(trim_padding(settings, header, buf))
        })
    }

    fn parse_headers(header: FrameHeader, mut buf: &'a [u8],
                     settings: ParserSettings) -> Result<Payload<'a>, Error> {
        buf = try!(trim_padding(settings, header, buf));
        let (buf, priority) = try!(Priority::parse(settings, buf));
        Ok(Payload::Headers {
            priority: priority,
            block: buf
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Priority {
    exclusive: bool,
    dependency: StreamIdentifier,
    weight: u8
}

impl Priority {
    #[inline]
    pub fn parse(settings: ParserSettings,
                 buf: &[u8]) -> Result<(&[u8], Option<Priority>), Error> {
        if settings.priority && buf.len() > 5 {
            Ok((&buf[5..], Some(Priority {
                // Most significant bit.
                exclusive: buf[0] & 0x80 != 0,
                dependency: StreamIdentifier::parse(buf),
                weight: buf[4]
            })))
        } else if settings.priority {
            Err(Error::Short)
        } else {
            Ok((buf, None))
        }
    }
}

// Settings are (u16, u32) in memory.
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct Setting {
    pub identifier: SettingIdentifier,
    value: u32
}

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum SettingIdentifier {
    HeaderTableSize = 0x1,
    EnablePush = 0x2,
    MaxConcurrentStreams = 0x3,
    InitialWindowSize = 0x4,
    MaxFrameSize = 0x5
}

fn trim_padding(settings: ParserSettings, header: FrameHeader,
                buf: &[u8]) -> Result<&[u8], Error> {
    if settings.padding {
        let pad_length = buf[0];
        if pad_length as u32 > header.length {
            Err(Error::TooMuchPadding(pad_length))
        } else {
            Ok(&buf[1..header.length as usize - pad_length as usize])
        }
    } else {
        Ok(buf)
    }
}

