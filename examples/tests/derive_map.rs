use packet_core::{OutgoingPacket, PacketDecode};
use packet_macros::Packet;
use serde_value::Value;

#[derive(Debug, Packet)]
#[packet(event = "m", map)]
struct InMap {
    name: String,
    score: i64,
    #[packet(rename = "tier-name")]
    tier: String,
    #[packet(default)]
    notes: String,
}

fn map_payload() -> Vec<Value> {
    let mut m = std::collections::BTreeMap::new();
    m.insert(Value::String("name".into()), Value::String("alice".into()));
    m.insert(Value::String("score".into()), Value::I64(42));
    m.insert(
        Value::String("tier-name".into()),
        Value::String("gold".into()),
    );
    vec![Value::Map(m.into_iter().collect())]
}

#[test]
fn map_mode_decodes_by_field_name() {
    let v = InMap::decode_payload(&map_payload()).unwrap();
    assert_eq!(v.name, "alice");
    assert_eq!(v.score, 42);
    assert_eq!(v.tier, "gold");
    assert_eq!(v.notes, "");
}

#[test]
fn map_mode_round_trips_via_outgoing() {
    let v = InMap::decode_payload(&map_payload()).unwrap();
    let out = v.to_values().unwrap();
    assert_eq!(out.len(), 2);
    assert!(matches!(&out[0], Value::String(s) if s == "m"));
    assert!(matches!(&out[1], Value::Map(_)));
}

#[test]
fn map_mode_fails_on_missing_required_field() {
    let mut m = std::collections::BTreeMap::new();
    m.insert(Value::String("name".into()), Value::String("alice".into()));
    let payload = vec![Value::Map(m.into_iter().collect())];
    let r = InMap::decode_payload(&payload);
    assert!(r.is_err());
}
