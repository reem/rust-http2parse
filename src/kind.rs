#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Data = 0,
    Headers = 1,
    Priority = 2,
    Reset = 3,
    Settings = 4,
    PushPromise = 5,
    Ping = 6,
    GoAway = 7,
    WindowUpdate = 8,
    Continuation = 9,
    Unregistered
}

impl Kind {
    pub fn new(byte: u8) -> Kind {
        return match byte {
            0 => Kind::Data,
            1 => Kind::Headers,
            2 => Kind::Priority,
            3 => Kind::Reset,
            4 => Kind::Settings,
            5 => Kind::PushPromise,
            6 => Kind::Ping,
            7 => Kind::GoAway,
            8 => Kind::WindowUpdate,
            9 => Kind::Continuation,
            _ => Kind::Unregistered
        }
    }
}

