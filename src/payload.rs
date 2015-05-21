use {StreamIdentifier};

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

