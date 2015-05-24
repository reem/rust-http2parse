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
            Kind::Data => {
                parse_payload_with_padding(settings, header, buf, |buf| {
                    Payload::Data {
                        data: buf
                    }
                })
            },
            _ => panic!("unimplemented")
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Priority {
    exclusive: bool,
    dependency: StreamIdentifier,
    weight: u8
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

