#![no_main]

use libfuzzer_sys::fuzz_target;
use packet_core::PacketDecode;
use packet_macros::Packet;

#[derive(Debug, Packet)]
#[packet(event = "fz", allow_extra)]
struct FuzzPacket {
    a: i64,
    b: Option<String>,
    #[packet(default)]
    c: i64,
}

fuzz_target!(|data: &[u8]| {
    if let Ok((values, _trailer)) = packet_core::wire::unpack_frame(data) {
        let _ = FuzzPacket::decode_payload(&values);
    }
});
