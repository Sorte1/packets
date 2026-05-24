#![no_main]

use libfuzzer_sys::fuzz_target;
use packet_core::PacketDecode;
use packet_macros::Packet;
use serde::Deserialize;
use serde_value::Value;
use std::io::Cursor;

#[derive(Debug, Packet)]
#[packet(event = "fz", allow_extra)]
struct FuzzPacket {
    a: i64,
    b: Option<String>,
    #[packet(default)]
    c: i64,
    #[packet(coerce(num_to_bool, str_num))]
    flag: bool,
}

fuzz_target!(|data: &[u8]| {
    let mut de = rmp_serde::Deserializer::new(Cursor::new(data));
    if let Ok(values) = Vec::<Value>::deserialize(&mut de) {
        let _ = FuzzPacket::decode_payload(&values);
    }
});
