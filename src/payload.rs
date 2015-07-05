use std::{slice, mem};
use {FrameHeader, StreamIdentifier, Error, Kind,
     ParserSettings, ErrorCode, SizeIncrement};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Payload<'a> {
    Data {
        data: &'a [u8]
    },
    Headers {
        priority: Option<Priority>,
        block: &'a [u8]
    },
    Priority(Priority),
    Reset(ErrorCode),
    Settings(&'a [Setting]),
    PushPromise {
        promised: StreamIdentifier,
        block: &'a [u8]
    },
    Ping(u64),
    GoAway {
        last: StreamIdentifier,
        error: ErrorCode,
        data: &'a [u8]
    },
    WindowUpdate(SizeIncrement),
    Continuation(&'a [u8]),
    Unregistered
}

const PRIORITY_BYTES: u32 = 5;
const PADDING_BYTES: u32 = 1;

impl<'a> Payload<'a> {
    pub fn parse(header: FrameHeader, mut buf: &'a [u8],
                 settings: ParserSettings) -> Result<Payload<'a>, Error> {
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

        buf = &buf[..header.length as usize];

        match header.kind {
            Kind::Data => Payload::parse_data(header, buf, settings),
            Kind::Headers => Payload::parse_headers(header, buf, settings),
            Kind::Priority => {
                let (_, priority) = try!(Priority::parse(settings, buf));
                Ok(Payload::Priority(priority.unwrap()))
            },
            Kind::Reset => Payload::parse_reset(header, buf),
            Kind::Settings => Payload::parse_settings(header, buf),
            Kind::Ping => Payload::parse_ping(header, buf),
            Kind::GoAway => Payload::parse_goaway(header, buf),
            Kind::WindowUpdate => Payload::parse_window_update(header, buf),
            Kind::PushPromise => Payload::parse_push_promise(header, buf, settings),
            Kind::Continuation => Ok(Payload::Continuation(buf)),
            Kind::Unregistered => Ok(Payload::Unregistered)
        }
    }

    #[inline]
    fn parse_data(header: FrameHeader, buf: &'a [u8],
                  settings: ParserSettings) -> Result<Payload<'a>, Error> {
        Ok(Payload::Data {
            data: try!(trim_padding(settings, header, buf))
        })
    }

    #[inline]
    fn parse_headers(header: FrameHeader, mut buf: &'a [u8],
                     settings: ParserSettings) -> Result<Payload<'a>, Error> {
        buf = try!(trim_padding(settings, header, buf));
        let (buf, priority) = try!(Priority::parse(settings, buf));
        Ok(Payload::Headers {
            priority: priority,
            block: buf
        })
    }

    #[inline]
    fn parse_reset(header: FrameHeader,
                   buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length < 4 {
            return Err(Error::PayloadLengthTooShort)
        }

        Ok(Payload::Reset(ErrorCode::parse(buf)))
    }

    #[inline]
    fn parse_settings(header: FrameHeader,
                      buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length % mem::size_of::<Setting>() as u32 != 0 {
            return Err(Error::PartialSettingLength)
        }

        Ok(Payload::Settings(
            unsafe {
                slice::from_raw_parts(
                    buf.as_ptr() as *const Setting,
                    buf[..header.length as usize].len() / mem::size_of::<Setting>())
            }
        ))
    }

    #[inline]
    fn parse_ping(header: FrameHeader,
                  buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length != 8 {
            return Err(Error::InvalidPayloadLength)
        }

        let payload = buf[..8].as_ptr() as *const u64;
        let data = unsafe { *payload };
        Ok(Payload::Ping(data))
    }

    #[inline]
    fn parse_goaway(header: FrameHeader,
                    buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length < 8 {
            return Err(Error::PayloadLengthTooShort)
        }

        let last = StreamIdentifier::parse(buf);
        let error = ErrorCode::parse(&buf[4..]);
        let rest = &buf[8..];

        Ok(Payload::GoAway {
            last: last,
            error: error,
            data: rest
        })
    }

    #[inline]
    fn parse_window_update(header: FrameHeader,
                           buf: &'a [u8]) -> Result<Payload<'a>, Error> {
        if header.length != 4 {
            return Err(Error::InvalidPayloadLength)
        }

        Ok(Payload::WindowUpdate(SizeIncrement::parse(buf)))
    }

    #[inline]
    fn parse_push_promise(header: FrameHeader, mut buf: &'a [u8],
                          settings: ParserSettings) -> Result<Payload<'a>, Error> {
        buf = try!(trim_padding(settings, header, buf));

        if buf.len() < 4 {
            return Err(Error::PayloadLengthTooShort)
        }

        let promised = StreamIdentifier::parse(buf);
        let block = &buf[4..];

        Ok(Payload::PushPromise {
             promised: promised,
             block: block
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Priority {
    exclusive: bool,
    dependency: StreamIdentifier,
    weight: u8
}

impl Priority {
    #[inline]
    pub fn parse(settings: ParserSettings,
                 buf: &[u8]) -> Result<(&[u8], Option<Priority>), Error> {
        if settings.priority {
            Ok((&buf[5..], Some(Priority {
                // Most significant bit.
                exclusive: buf[0] & 0x7F != buf[0],
                dependency: StreamIdentifier::parse(buf),
                weight: buf[4]
            })))
        } else {
            Ok((buf, None))
        }
    }
}

// Settings are (u16, u32) in memory.
#[repr(packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    identifier: u16,
    value: u32
}

impl Setting {
    pub fn identifier(&self) -> Option<SettingIdentifier> {
        match self.identifier {
            0x1 => Some(SettingIdentifier::HeaderTableSize),
            0x2 => Some(SettingIdentifier::EnablePush),
            0x3 => Some(SettingIdentifier::MaxConcurrentStreams),
            0x4 => Some(SettingIdentifier::InitialWindowSize),
            0x5 => Some(SettingIdentifier::MaxFrameSize),
            _ => None
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

