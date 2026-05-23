use packet_core::sentinel::{
    EmptySeqOrDefault, EmptyStringOrDefault, NegOneOrDefault, NullishOrDefault,
};
use serde::Deserialize;
use serde_value::Value;

fn round_trip<T: serde::de::DeserializeOwned>(value: Value) -> T {
    T::deserialize(value).unwrap()
}

#[test]
fn neg_one_decodes_to_default() {
    let s: NegOneOrDefault<String> = round_trip(Value::I64(-1));
    assert_eq!(*s, String::default());
}

#[test]
fn neg_one_passes_through_non_sentinel() {
    let n: NegOneOrDefault<i64> = round_trip(Value::I64(42));
    assert_eq!(*n, 42);
}

#[test]
fn empty_string_decodes_to_default() {
    let n: EmptyStringOrDefault<i64> = round_trip(Value::String(String::new()));
    assert_eq!(*n, 0);
}

#[test]
fn empty_string_passes_through() {
    let s: EmptyStringOrDefault<String> = round_trip(Value::String("hi".into()));
    assert_eq!(*s, "hi");
}

#[test]
fn empty_seq_decodes_to_default() {
    #[derive(Deserialize, Debug, PartialEq, Default)]
    struct Bag {
        a: i64,
    }
    let b: EmptySeqOrDefault<Bag> = round_trip(Value::Seq(Vec::new()));
    assert_eq!(*b, Bag::default());
}

#[test]
fn nullish_decodes_unit_option_none_and_false_to_default() {
    let a: NullishOrDefault<i64> = round_trip(Value::Unit);
    let b: NullishOrDefault<i64> = round_trip(Value::Option(None));
    let c: NullishOrDefault<i64> = round_trip(Value::Bool(false));
    assert_eq!(*a, 0);
    assert_eq!(*b, 0);
    assert_eq!(*c, 0);
}
