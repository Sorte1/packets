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
fn lossless_rejects_negative_float_to_u64() {
    let v = Value::F64(-3.5);
    let flags = CoerceFlags {
        lossless: true,
        ..Default::default()
    };
    let r: Result<u64, _> = decode_field_coerce("e", 0, "f", flags, Some(&v));
    assert!(r.is_err(), "lossless mode must reject negative f64 -> u64");
}

#[test]
fn lossless_accepts_exact_integer_float() {
    let v = Value::F64(42.0);
    let flags = CoerceFlags {
        lossless: true,
        ..Default::default()
    };
    let n: i64 = decode_field_coerce("e", 0, "f", flags, Some(&v)).unwrap();
    assert_eq!(n, 42);
}

#[test]
fn lossless_rejects_fractional_to_int() {
    let v = Value::F64(3.5);
    let flags = CoerceFlags {
        lossless: true,
        ..Default::default()
    };
    let r: Result<i64, _> = decode_field_coerce("e", 0, "f", flags, Some(&v));
    assert!(r.is_err());
}

#[test]
fn lossy_accepts_what_lossless_rejects() {
    let v = Value::F64(-3.5);
    let flags = CoerceFlags {
        lossless: false,
        ..Default::default()
    };
    let r: Result<u64, _> = decode_field_coerce("e", 0, "f", flags, Some(&v));
    assert!(r.is_ok(), "lossy mode should let the truncated cast through");
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
