use packet_core::decode::{decode_field_coerce, CoerceFlags};
use serde_value::Value;

#[test]
fn str_num_parses_int_string() {
    let v = Value::String("42".into());
    let n: i64 = decode_field_coerce(
        "e",
        0,
        "f",
        CoerceFlags {
            str_num: true,
            ..Default::default()
        },
        Some(&v),
    )
    .unwrap();
    assert_eq!(n, 42);
}

#[test]
fn str_num_parses_float_string() {
    let v = Value::String("3.5".into());
    let f: f64 = decode_field_coerce(
        "e",
        0,
        "f",
        CoerceFlags {
            str_num: true,
            ..Default::default()
        },
        Some(&v),
    )
    .unwrap();
    assert_eq!(f, 3.5);
}

#[test]
fn str_num_off_by_default_so_string_decode_fails() {
    let v = Value::String("42".into());
    let r: Result<i64, _> = decode_field_coerce("e", 0, "f", CoerceFlags::default(), Some(&v));
    assert!(r.is_err());
}

#[test]
fn str_num_decodes_true_false_to_bool() {
    let t = Value::String("true".into());
    let b: bool = decode_field_coerce(
        "e",
        0,
        "f",
        CoerceFlags {
            str_num: true,
            ..Default::default()
        },
        Some(&t),
    )
    .unwrap();
    assert!(b);
}
