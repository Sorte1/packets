Readme is ai generated

# packets

Derive macros for Krunker WebSocket packets.

Krunker sends each message as an event name followed by a flat positional array:

```
["k",  42, 1.0, 5.0, ..., 33, 0]   ← player update
["f",  2]                            ← failed input
["po"]                               ← ping (no payload)
```

This crate lets you describe those packets as plain Rust structs and generates the encode/decode boilerplate for you.

## Crates

- **`packet_core`** — runtime traits and decode helpers (`PacketMeta`, `PacketDecode`, `OutgoingPacket`, `OutgoingFields`, `OutgoingEventEnum`).
- **`packet_macros`** — proc macros that emit the `impl` blocks: `Packet`, `OutgoingFields`, `EventEnum`, `OutgoingEventEnum`.

They are split because Rust proc macros cannot be used by the same crate that defines them.

## Quick taste

```rust
use packet_macros::Packet;

#[derive(Debug, Packet)]
#[packet(event = "f")]
pub struct FailedInput {
    pub issue: i64,   // payload[0]
}
```

That's it — `FailedInput` now decodes from a payload array and encodes back to one.

A more interesting example, with a chunked sub-array:

```rust
#[derive(Debug, Packet)]
#[packet(event = "k", allow_extra)]
pub struct PlayerUpdate {
    #[packet(chunks(PlayerUpdateData, 13))]
    pub updates: Vec<PlayerUpdateData>,
    pub rate: i64,
    pub timestamp: i64,
}
```

And a top-level dispatcher:

```rust
#[derive(Debug, EventEnum)]
pub enum Event {
    FailedInput(FailedInput),
    PlayerUpdate(PlayerUpdate),

    #[event(unknown)]
    Unknown(String, Vec<Value>),
}
```

`Event::from_event_name(name, payload)` matches on each variant's `EVENT_NAME` and decodes for you.

## Examples

Runnable binaries live in [`examples/`](examples/):

- `incoming_decode` — `scalar_as_seq`, `Option` fields, and `chunks(T, N)`
- `outgoing_encode` — empty packets, nested fields, and `coerce(bool_to_num)`
- `event_dispatch` — the `EventEnum` dispatch loop
- `proxy_intercept` — `OutgoingEventEnum` decode-mutate-reencode round-trip

Run any of them with `cargo run -p packet_examples --bin <name>`.
