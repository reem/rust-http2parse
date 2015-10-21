use {Payload, Error, Flag, Kind, StreamIdentifier, FRAME_HEADER_BYTES};

#[cfg(feature = "random")]
use rand::{Rand, Rng};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frame<'a> {
    pub header: FrameHeader,
    pub payload: Payload<'a>
}

impl<'a> Frame<'a> {
    pub fn parse(header: FrameHeader, buf: &[u8]) -> Result<Frame, Error> {
        Ok(Frame {
            header: header,
            payload: try!(Payload::parse(header, buf))
        })
    }

    /// Encodes this Frame into a buffer.
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        self.header.encode(buf);
        self.payload.encode(&mut buf[FRAME_HEADER_BYTES..]) + FRAME_HEADER_BYTES
    }

    /// How many bytes this Frame will use in a buffer when encoding.
    pub fn encoded_len(&self) -> usize {
        FRAME_HEADER_BYTES + self.payload.encoded_len()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}

impl FrameHeader {
    #[inline]
    pub fn parse(buf: &[u8]) -> Result<FrameHeader, Error> {
        if buf.len() < FRAME_HEADER_BYTES {
            return Err(Error::Short);
        }

        Ok(FrameHeader {
            length: ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | buf[2] as u32,
            kind: Kind::new(buf[3]),
            flag: try!(Flag::new(buf[4]).map_err(|()| { Error::BadFlag(buf[4]) })),
            id: StreamIdentifier::parse(&buf[5..])
        })
    }

    #[inline]
    pub fn encode(&self, buf: &mut [u8]) {
        ::encode_u24(buf, self.length);
        buf[3] = self.kind.encode();
        buf[4] = self.flag.bits();
        self.id.encode(&mut buf[5..]);
    }

    #[inline]
    #[cfg(feature = "random")]
    fn rand_for_payload<R: Rng>(rng: &mut R, payload: &Payload) -> FrameHeader {
        let len = payload.encoded_len();

        if len > 1 << 24 {
            panic!("Overlong payload for testing.")
        }

        let flags = Flag::empty()
            // if the payload has priority add the priority header.
            | if let Some(_) = payload.priority() { Flag::priority() } else { Flag::empty() };

        FrameHeader {
            length: len as u32,
            kind: payload.kind(),
            flag: flags,
            ..rng.gen()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "random")]
pub fn rand_for_payload(payload: &Payload) -> FrameHeader {
    FrameHeader::rand_for_payload(&mut ::rand::thread_rng(), payload)
}

#[cfg(feature = "random")]
impl Rand for FrameHeader {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        FrameHeader {
            length: rng.gen_range(0, 1 << 24),
            kind: Kind::new(rng.gen_range(0, 9)),
            flag: *rng.choose(&[Flag::padded() | Flag::priority()])
                    .unwrap_or(&Flag::empty()),
            id: StreamIdentifier(rng.gen_range(0, 1 << 31))
        }
    }
}

#[cfg(feature = "random")]
impl Rand for Frame<'static> {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        let payload = rng.gen::<Payload>();
        let header = FrameHeader::rand_for_payload(rng, &payload);

        Frame {
            header: header,
            payload: payload
        }
    }
}

#[cfg(test)]
mod test {
    use {Kind, Flag, FrameHeader, StreamIdentifier};
    #[cfg(feature = "random")]
    use {Frame};

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

    #[cfg(feature = "random")]
    #[test]
    fn test_frame_header_encoding() {
        fn roundtrip(header: FrameHeader) {
            let buf = &mut [0; 9];
            header.encode(buf);

            assert_eq!(header, FrameHeader::parse(&*buf).unwrap())
        }

        for _ in 0..100 {
            roundtrip(::rand::random())
        }
    }

    #[cfg(feature = "random")]
    #[test]
    fn test_frame_encoding() {
        fn roundtrip(buf: &mut [u8], frame: Frame) {
            let _ = frame.encode(buf);

            assert_eq!(
                frame,
                Frame::parse(FrameHeader::parse(&buf[..9]).unwrap(),
                             &buf[9..]).unwrap())
        }

        let mut buf = vec![0; 5000];
        for _ in 0..1000 {
            roundtrip(&mut buf, ::rand::random())
        }
    }

    #[cfg(not(feature = "random"))]
    #[test]
    fn no_frame_encoding_test_because_no_rand() {}

    #[bench]
    #[cfg(feature = "random")]
    fn bench_frame_parse(b: &mut ::test::Bencher) {
        // Each iter = 5 frames
        let frames = vec![::rand::random::<Frame>(); 5];
        let bufs = frames.iter().map(|frame| {
            let mut buf = vec![0; 2000];
            frame.encode(&mut buf);
            buf
        }).collect::<Vec<_>>();

        b.bytes = frames.iter().map(|frame| frame.encoded_len() as u64)
            .fold(0, |a, b| a + b);

        b.iter(|| {
            for buf in &bufs {
                Frame::parse(FrameHeader::parse(&buf[..9]).unwrap(),
                             &buf[9..]).unwrap();
            }
        });
    }

    #[bench]
    #[cfg(feature = "random")]
    fn bench_frame_encode(b: &mut ::test::Bencher) {
        let frames = vec![::rand::random::<Frame>(); 5];

        b.bytes = frames.iter().map(|frame| frame.encoded_len() as u64)
            .fold(0, |a, b| a + b);

        let mut buf = vec![0; 2000];
        b.iter(|| {
            for frame in &frames {
                frame.encode(&mut buf);
                ::test::black_box(&buf);
            }
        });
    }

    #[bench]
    fn bench_frame_header_parse(b: &mut ::test::Bencher) {
        b.bytes = ::FRAME_HEADER_BYTES as u64;

        b.iter(|| {
            let mut buf = &[
                0x1, 0x2, 0x3, // length
                0x4, // type/kind
                0x1, // flags
                0x6, 0x7, 0x8, 0x9 // reserved bit + stream identifier
            ];

            // Prevent constant propagation.
            buf = ::test::black_box(buf);

            // Prevent dead code elimination.
            ::test::black_box(FrameHeader::parse(buf).unwrap());
        });
    }
}

