use packet_core::error::{DecodeError, DecodeErrorKind, DecodePath, PathSegment, ValueKind};

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

#[test]
fn decode_path_renders_event_field_chunk_sub() {
    let path = DecodePath(vec![
        PathSegment::Event("k"),
        PathSegment::Field {
            name: "updates",
            idx: 0,
        },
        PathSegment::Chunk(3),
        PathSegment::Sub("grapple"),
    ]);
    assert_eq!(path.to_string(), "k.updates[3].grapple");
}

#[test]
fn decode_error_with_path_displays_path_prefix() {
    let mut err: DecodeError = DecodeErrorKind::Missing {
        event: "k",
        field: "grapple",
        idx: 11,
    }
    .into();
    err.prepend(PathSegment::Chunk(3));
    err.prepend(PathSegment::Field {
        name: "updates",
        idx: 0,
    });
    err.prepend(PathSegment::Event("k"));
    let s = err.to_string();
    assert!(s.starts_with("k.updates[3]"), "path prefix missing: {s}");
}

#[test]
fn chunked_field_failure_includes_chunk_index_in_path() {
    use packet_core::decode::decode_chunks;
    use serde::Deserialize;
    use serde_value::Value;

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Item {
        a: i64,
        b: i64,
    }

    let raw = Value::Seq(vec![
        Value::I64(1),
        Value::I64(2),
        Value::String("oops".into()),
        Value::I64(4),
    ]);
    let err = decode_chunks::<Item>("evt", 0, "items", 2, Some(&raw)).unwrap_err();
    let s = err.to_string();
    assert!(s.contains("[1]"), "expected chunk index 1 in path, got: {s}");
}
