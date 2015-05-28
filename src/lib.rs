#![cfg_attr(test, deny(warnings))]
#![allow(non_upper_case_globals)]
//#![deny(missing_docs)]

//! # http2parse
//!
//! An HTTP2 frame parser.
//!

#[macro_use]
extern crate bitflags;

const FRAME_HEADER_BYTES: usize = 9;

pub use kind::Kind;
pub use flag::Flag;
pub use frame::{Frame, FrameHeader};
pub use payload::{Payload, Priority, Setting, SettingIdentifier};

mod kind;
mod flag;
mod payload;
mod frame;

// contains type and flags
// Bitflags?
// things that can go wrong during parsing
#[derive(Copy, Clone, Debug)]
pub enum Error {
    /// A full frame header was not passed.
    Short,

    /// An unsupported value was set for the flag value.
    BadFlag(u8),

    /// An unsupported value was set for the frame kind.
    BadKind(u8),

    /// The padding length was larger than the frame-header-specified
    /// length of the payload.
    TooMuchPadding(u8),

    /// The payload length specified by the frame header was shorter than
    /// necessary for the parser settings specified and the frame type.
    ///
    /// This happens if, for instance, the priority flag is set and the
    /// header length is shorter than a stream dependency.
    ///
    /// `PayloadLengthTooShort` should be treated as a protocol error.
    PayloadLengthTooShort,

    /// The payload length specified by the frame header of a settings frame
    /// was not a round multiple of the size of a single setting.
    PartialSettingLength,

    /// The payload length specified by the frame header of a ping frame
    /// was not 8, the length of the opaque data of a ping frame.
    InvalidLengthForPing
}

#[derive(Copy, Clone, Debug)]
pub struct ParserSettings {
    padding: bool,
    priority: bool
}

#[derive(Copy, Clone, Debug)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse(buf: &[u8]) -> StreamIdentifier {
        StreamIdentifier(
            ((buf[0] as u32) << 24) |
            ((buf[1] as u32) << 16) |
            ((buf[2] as u32) << 8) |
             (buf[3] as u32) |
             // Clear the most significant bit.
             (1 << 31 as u32)
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub fn parse(buf: &[u8]) -> ErrorCode {
        ErrorCode(
            ((buf[0] as u32) << 24) |
            ((buf[1] as u32) << 16) |
            ((buf[2] as u32) << 8) |
             (buf[3] as u32))
    }
}

