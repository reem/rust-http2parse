#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

    pub fn encode(&self) -> u8 {
        match *self {
            Kind::Data => 0,
            Kind::Headers => 1,
            Kind::Priority => 2,
            Kind::Reset => 3,
            Kind::Settings => 4,
            Kind::PushPromise => 5,
            Kind::Ping => 6,
            Kind::GoAway => 7,
            Kind::WindowUpdate => 8,
            Kind::Continuation => 9,
            Kind::Unregistered => 255
        }
    }
}

#[test]
fn test_encode() {
    for n in 0..10 {
        assert_eq!(Kind::new(n), Kind::new(Kind::new(n).encode()));
    }
}

