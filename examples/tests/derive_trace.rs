use packet_core::trace::{decode_payload_traced, FieldDecodeKind};
use packet_macros::Packet;
use serde_value::Value;

#[derive(Debug, Packet)]
#[packet(event = "trc")]
struct Traced {
    a: i64,
    b: Option<i64>,
    #[packet(default)]
    c: String,
}

#[test]
fn trace_records_one_entry_per_decoded_field() {
    let payload = vec![Value::I64(7), Value::I64(8), Value::String("hi".into())];
    let (result, trace) = decode_payload_traced::<Traced>(&payload);
    let v = result.unwrap();
    assert_eq!(v.a, 7);
    assert_eq!(v.b, Some(8));
    assert_eq!(v.c, "hi");
    assert_eq!(trace.fields.len(), 3);
    assert_eq!(trace.fields[0].field, "a");
    assert_eq!(trace.fields[0].kind, FieldDecodeKind::Direct);
    assert_eq!(trace.fields[1].kind, FieldDecodeKind::Direct);
}

#[test]
fn trace_records_default_when_field_missing() {
    let payload = vec![Value::I64(7), Value::I64(8)];
    let (result, trace) = decode_payload_traced::<Traced>(&payload);
    let v = result.unwrap();
    assert_eq!(v.c, "");
    let last = trace.fields.last().unwrap();
    assert_eq!(last.field, "c");
    assert_eq!(last.kind, FieldDecodeKind::Default);
}

#[test]
fn trace_records_optional_none_when_unit() {
    let payload = vec![Value::I64(7), Value::Option(None)];
    let (result, trace) = decode_payload_traced::<Traced>(&payload);
    let v = result.unwrap();
    assert!(v.b.is_none());
    assert!(trace
        .fields
        .iter()
        .any(|f| f.field == "b" && f.kind == FieldDecodeKind::OptionalNone));
}
