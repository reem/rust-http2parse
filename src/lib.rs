#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, feature(test))]
#![allow(non_upper_case_globals)]
// #![deny(missing_docs)]

//! # http2parse
//!
//! An HTTP2 frame parser.
//!

#[macro_use]
extern crate bitflags;
extern crate byteorder;

#[cfg(test)]
extern crate test;
#[cfg(any(test, feature = "random"))]
extern crate rand;

const FRAME_HEADER_BYTES: usize = 9;

pub use kind::Kind;
pub use flag::Flag;
pub use frame::{Frame, FrameHeader};
pub use payload::{Payload, Priority, Setting, SettingIdentifier};

use byteorder::ByteOrder;

mod kind;
mod flag;
mod payload;
mod frame;

/// Errors that can occur during parsing an HTTP/2 frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

    /// The payload length specified by the frame header was not the
    /// value necessary for the specific frame type.
    InvalidPayloadLength
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct ParserSettings {
    padding: bool,
    priority: bool
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse(buf: &[u8]) -> StreamIdentifier {
        StreamIdentifier(
            byteorder::BigEndian::read_u32(buf) & ((1 << 31) - 1)
        )
    }

    pub fn encode(&self, buf: &mut [u8]) {
        encode_32u(buf, self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub u32);

pub enum HttpError {
    Protocol,
    Internal,
    FlowControlError,
    SettingsTimeout,
}

impl ErrorCode {
    pub fn parse(buf: &[u8]) -> ErrorCode {
        ErrorCode(byteorder::BigEndian::read_u32(buf))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SizeIncrement(pub u32);

impl SizeIncrement {
    pub fn parse(buf: &[u8]) -> SizeIncrement {
        SizeIncrement(byteorder::BigEndian::read_u32(buf))
    }
}

#[inline(always)]
fn encode_24u(buf: &mut [u8], val: u32) {
    buf[0] = (val >> 16) as u8;
    buf[1] = (val >> 8) as u8;
    buf[2] = val as u8;
}

#[inline(always)]
fn encode_32u(buf: &mut [u8], val: u32) {
    buf[0] = (val >> 24) as u8;
    buf[1] = (val >> 16) as u8;
    buf[2] = (val >> 8) as u8;
    buf[3] = val as u8;
}

#[test]
fn test_stream_id_ignores_highest_bit() {
    let raw1 = [0x7F, 0xFF, 0xFF, 0xFF];
    let raw2 = [0xFF, 0xFF, 0xFF, 0xFF];

    assert_eq!(
        StreamIdentifier::parse(&raw1),
        StreamIdentifier::parse(&raw2));
}

