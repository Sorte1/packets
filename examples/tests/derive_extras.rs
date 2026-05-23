use packet_core::{OutgoingPacket, PacketDecode};
use packet_macros::Packet;
use serde_value::Value;

#[derive(Debug, Packet)]
#[packet(event = "x")]
struct WithExtras {
    a: i64,
    #[packet(extras)]
    rest: Vec<Value>,
}

#[test]
fn extras_collects_trailing_payload() {
    let payload = vec![
        Value::I64(1),
        Value::I64(2),
        Value::String("third".into()),
    ];
    let v = WithExtras::decode_payload(&payload).unwrap();
    assert_eq!(v.a, 1);
    assert_eq!(v.rest.len(), 2);
    assert!(matches!(v.rest[0], Value::I64(2)));
}

#[test]
fn extras_empty_when_no_trailing_values() {
    let payload = vec![Value::I64(1)];
    let v = WithExtras::decode_payload(&payload).unwrap();
    assert_eq!(v.a, 1);
    assert!(v.rest.is_empty());
}

#[test]
fn extras_round_trip_via_outgoing() {
    let payload = vec![
        Value::I64(1),
        Value::I64(2),
        Value::String("third".into()),
    ];
    let v = WithExtras::decode_payload(&payload).unwrap();
    let out = v.to_values().unwrap();
    assert_eq!(out.len(), 4);
    assert!(matches!(&out[0], Value::String(s) if s == "x"));
}
