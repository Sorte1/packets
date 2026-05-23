use packet_core::fmt::pretty;
use serde_value::Value;

#[test]
fn scalar_int_renders_with_kind_suffix() {
    assert_eq!(pretty(&Value::I64(42), 80), "42i64");
    assert_eq!(pretty(&Value::U8(7), 80), "7u8");
    assert_eq!(pretty(&Value::F64(3.5), 80), "3.5f64");
}

#[test]
fn string_escapes_and_quotes() {
    assert_eq!(pretty(&Value::String("hi\n".into()), 80), r#""hi\n""#);
}

#[test]
fn unit_and_option_render() {
    assert_eq!(pretty(&Value::Unit, 80), "()");
    assert_eq!(pretty(&Value::Option(None), 80), "None");
    assert_eq!(
        pretty(&Value::Option(Some(Box::new(Value::I64(1)))), 80),
        "Some(1i64)"
    );
}

#[test]
fn seq_truncates_when_over_budget() {
    let big: Vec<Value> = (0..20).map(Value::I64).collect();
    let s = pretty(&Value::Seq(big), 40);
    assert!(s.starts_with("["), "expected leading bracket: {s}");
    assert!(s.contains("…+"), "expected truncation marker: {s}");
}

#[test]
fn map_keys_sorted_for_determinism() {
    let entries = vec![
        (Value::String("b".into()), Value::I64(2)),
        (Value::String("a".into()), Value::I64(1)),
    ];
    let val = Value::Map(entries.into_iter().collect());
    let s = pretty(&val, 80);
    let a = s.find("\"a\"").unwrap();
    let b = s.find("\"b\"").unwrap();
    assert!(a < b, "keys not sorted: {s}");
}
