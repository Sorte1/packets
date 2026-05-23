use packet_core::error::{DecodeError, DecodeErrorKind, ValueKind};

#[test]
fn missing_field_display_includes_event_and_field() {
    let err: DecodeError = DecodeErrorKind::Missing {
        event: "k",
        field: "updates",
        idx: 0,
    }
    .into();
    let s = err.to_string();
    assert!(s.contains("k"), "missing event name: {s}");
    assert!(s.contains("updates"), "missing field name: {s}");
    assert!(s.contains("#0"), "missing field index: {s}");
}

#[test]
fn type_mismatch_display_shows_expected_and_got() {
    let err: DecodeError = DecodeErrorKind::TypeMismatch {
        event: "start",
        field: "timer",
        idx: 0,
        expected: "String",
        got: ValueKind::I64,
    }
    .into();
    let s = err.to_string();
    assert!(s.contains("String"), "missing expected: {s}");
    assert!(s.contains("i64"), "missing got kind: {s}");
}

#[test]
fn decode_error_converts_to_anyhow() {
    let err: DecodeError = DecodeErrorKind::Missing {
        event: "k",
        field: "updates",
        idx: 0,
    }
    .into();
    let anyhow_err: anyhow::Error = err.into();
    assert!(anyhow_err.to_string().contains("updates"));
}
