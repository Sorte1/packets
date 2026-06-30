# krunker_packet_macros

Derive macros for describing [Krunker](https://krunker.io) WebSocket packets as
plain Rust structs.

> Published on crates.io as `krunker_packet_macros`; imported in Rust code as `packet_macros`.

Krunker sends each message as an event name followed by a flat positional array.
This crate lets you describe a packet as a struct and generates the encode/decode
boilerplate against the traits in
[`krunker_packet_core`](https://crates.io/crates/krunker_packet_core) (imported as
`packet_core`), which you also need as a dependency (a proc-macro crate cannot be
used by the crate that defines it).

```toml
[dependencies]
krunker_packet_core = "0.1"
krunker_packet_macros = "0.1"
```

```rust
use packet_macros::Packet;

#[derive(Debug, Packet)]
#[packet(event = "f")]
pub struct FailedInput {
    pub issue: i64, // payload[0]
}
```

`FailedInput` now decodes from a payload array and encodes back to one.

## Derives

- `Packet` — encode/decode for a single event packet.
- `OutgoingFields` — encode a struct as a positional field group.
- `EventEnum` — a top-level dispatcher with `from_event_name`, plus
  `OutgoingPacket`.
- `OutgoingEventEnum` — frame-level encode for an outgoing event enum.

See the [workspace README](https://github.com/Sorte1/packets) for chunked
sub-arrays, dispatch loops, and runnable examples.

## License

MIT
