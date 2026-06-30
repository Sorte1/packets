# krunker_packet_core

Runtime traits and msgpack decode helpers for [Krunker](https://krunker.io) WebSocket packets.

> Published on crates.io as `krunker_packet_core`; imported in Rust code as `packet_core`.

Krunker sends each message as an event name followed by a flat positional array:

```
["k",  42, 1.0, 5.0, ..., 33, 0]   ← player update
["f",  2]                            ← failed input
["po"]                               ← ping (no payload)
```

This crate provides the traits and decode helpers that describe those packets:
`PacketMeta`, `PacketDecode`, `OutgoingPacket`, `OutgoingFields`, and
`OutgoingEventEnum`, plus the wire framing used to split and stitch the
trailing bytes Krunker appends to each frame.

You usually pair it with [`krunker_packet_macros`](https://crates.io/crates/krunker_packet_macros)
(imported as `packet_macros`), which derives the `impl` blocks for these traits from
plain Rust structs. They are split because a proc-macro crate cannot be used by the
crate that defines it.

## Features

- `tracing` — emit `tracing` spans/events around decoding (off by default).

See the [workspace README](https://github.com/Sorte1/packets) for runnable examples.

## License

MIT
