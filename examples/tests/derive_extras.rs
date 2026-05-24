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

#[derive(Debug, Packet)]
#[packet(event = "y")]
struct WithAt {
    a: i64,
    #[packet(at = 3)]
    far: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "g", tolerate_garbage)]
struct Lenient {
    a: i64,
    b: i64,
    c: String,
}

#[test]
fn tolerate_garbage_defaults_a_field_with_wrong_type() {
    let payload = vec![
        Value::I64(7),
        Value::String("nope".into()),
        Value::String("ok".into()),
    ];
    let v = Lenient::decode_payload(&payload).unwrap();
    assert_eq!(v.a, 7);
    assert_eq!(v.b, 0);
    assert_eq!(v.c, "ok");
}

#[test]
fn tolerate_garbage_allows_trailing_garbage_values() {
    let payload = vec![
        Value::I64(7),
        Value::I64(8),
        Value::String("nine".into()),
        Value::String("ten".into()),
    ];
    let v = Lenient::decode_payload(&payload).unwrap();
    assert_eq!(v.a, 7);
    assert_eq!(v.b, 8);
    assert_eq!(v.c, "nine");
}

fn default_minus_one() -> i64 {
    -1
}

#[derive(Debug, Packet)]
#[packet(event = "z")]
struct WithDefault {
    a: i64,
    #[packet(default_with = "default_minus_one")]
    b: i64,
}

#[test]
fn default_with_fires_when_field_missing() {
    let v = WithDefault::decode_payload(&[Value::I64(7)]).unwrap();
    assert_eq!(v.a, 7);
    assert_eq!(v.b, -1);
}

#[test]
fn default_with_skipped_when_field_present() {
    let v = WithDefault::decode_payload(&[Value::I64(7), Value::I64(9)]).unwrap();
    assert_eq!(v.b, 9);
}

#[test]
fn at_skips_to_explicit_index() {
    let payload = vec![
        Value::I64(10),
        Value::I64(99),
        Value::I64(99),
        Value::I64(42),
    ];
    let v = WithAt::decode_payload(&payload).unwrap();
    assert_eq!(v.a, 10);
    assert_eq!(v.far, 42);
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
