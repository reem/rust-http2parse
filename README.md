# http2parse

> An HTTP2 frame parser.

## Overview

A parser for HTTP2 frames. http2parse implements fast decoding
of all HTTP2 frames into an efficient typed representation with
no copying of frame payload data.

http2parse does not implement HPACK encoding and decoding,
which will likely end up in a different crate.

## Usage

Use the crates.io repository; add this to your `Cargo.toml` along
with the rest of your dependencies:

```toml
[dependencies]
http2parse = "0"
```

## Author

[Jonathan Reem](https://medium.com/@jreem) is the primary author and maintainer of http2parse.

## License

MIT

