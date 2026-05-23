use packet_core::oneof::{OneOf2, OneOf3};
use serde::Deserialize;
use serde_value::Value;

#[test]
fn one_of_2_picks_first_match() {
    let v: OneOf2<i64, String> = OneOf2::deserialize(Value::I64(7)).unwrap();
    assert!(matches!(v, OneOf2::A(7)));
}

#[test]
fn one_of_2_falls_back_to_second() {
    let v: OneOf2<i64, String> = OneOf2::deserialize(Value::String("hi".into())).unwrap();
    assert!(matches!(v, OneOf2::B(s) if s == "hi"));
}

#[test]
fn one_of_2_fails_when_neither_matches() {
    let r: Result<OneOf2<i64, String>, _> = OneOf2::deserialize(Value::Bool(true));
    assert!(r.is_err());
}

#[test]
fn one_of_3_walks_through_variants() {
    let v: OneOf3<i64, bool, String> = OneOf3::deserialize(Value::Bool(true)).unwrap();
    assert!(matches!(v, OneOf3::B(true)));
}
