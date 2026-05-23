use packet_core::decode::decode_var_chunks;
use serde::Deserialize;
use serde_value::Value;

#[derive(Debug, Deserialize, PartialEq)]
struct Row {
    a: i64,
    b: i64,
}

#[test]
fn var_chunks_parses_length_prefixed_records() {
    let payload = Value::Seq(vec![
        Value::U64(2),
        Value::I64(10),
        Value::I64(20),
        Value::U64(2),
        Value::I64(30),
        Value::I64(40),
    ]);
    let rows: Vec<Row> = decode_var_chunks("e", 0, "rows", Some(&payload)).unwrap();
    assert_eq!(
        rows,
        vec![Row { a: 10, b: 20 }, Row { a: 30, b: 40 }]
    );
}

#[test]
fn var_chunks_supports_value_payloads() {
    let payload = Value::Seq(vec![
        Value::U64(1),
        Value::I64(99),
        Value::U64(3),
        Value::I64(1),
        Value::I64(2),
        Value::I64(3),
    ]);
    let chunks: Vec<Vec<Value>> = decode_var_chunks("e", 0, "rows", Some(&payload)).unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].len(), 1);
    assert_eq!(chunks[1].len(), 3);
}

#[test]
fn var_chunks_errors_when_declared_length_overruns() {
    let payload = Value::Seq(vec![Value::U64(5), Value::I64(1), Value::I64(2)]);
    let r: Result<Vec<Vec<Value>>, _> = decode_var_chunks("e", 0, "rows", Some(&payload));
    assert!(r.is_err());
}

#[test]
fn var_chunks_errors_when_first_value_is_not_numeric() {
    let payload = Value::Seq(vec![Value::String("oops".into()), Value::I64(1)]);
    let r: Result<Vec<Vec<Value>>, _> = decode_var_chunks("e", 0, "rows", Some(&payload));
    assert!(r.is_err());
}
