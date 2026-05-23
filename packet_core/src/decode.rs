use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_value::Value;

use crate::error::{DecodeError, DecodeErrorKind, PathSegment};

pub fn scalar_as_seq(payload: &[Value]) -> Vec<Value> {
    match payload {
        [single] => match single {
            Value::Seq(seq) => seq.clone(),
            other => vec![other.clone()],
        },
        many => many.to_vec(),
    }
}

pub fn seq_payload(payload: &[Value], wrap_scalar: bool) -> Vec<Value> {
    if wrap_scalar {
        scalar_as_seq(payload)
    } else {
        payload.to_vec()
    }
}

pub fn decode_field<T: DeserializeOwned>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
    raw: Option<&Value>,
) -> Result<T, DecodeError> {
    let value = raw.ok_or_else(|| {
        DecodeError::from(DecodeErrorKind::Missing {
            event,
            field: field_name,
            idx: field_index,
        })
    })?;

    crate::deserialize_value(value.clone()).map_err(|err| {
        DecodeError::custom(format!(
            "event {:?} field #{} {:?}: {}",
            event, field_index, field_name, err
        ))
    })
}

pub fn decode_field_with_default<T: DeserializeOwned + Default>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
    raw: Option<&Value>,
) -> Result<T, DecodeError> {
    match raw {
        Some(value) => crate::deserialize_value(value.clone()).map_err(|err| {
            DecodeError::custom(format!(
                "event {:?} field #{} {:?}: {}",
                event, field_index, field_name, err
            ))
        }),
        None => Ok(T::default()),
    }
}

pub fn decode_chunks<T: DeserializeOwned>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
    chunk_size: usize,
    raw: Option<&Value>,
) -> Result<Vec<T>, DecodeError> {
    let value = raw.ok_or_else(|| {
        DecodeError::from(DecodeErrorKind::Missing {
            event,
            field: field_name,
            idx: field_index,
        })
    })?;

    let seq = match value {
        Value::Seq(seq) => seq,
        other => {
            return Err(DecodeError::from(DecodeErrorKind::TypeMismatch {
                event,
                field: field_name,
                idx: field_index,
                expected: "seq",
                got: crate::error::ValueKind::of(other),
            }));
        }
    };

    if seq.len() % chunk_size != 0 {
        return Err(DecodeError::from(DecodeErrorKind::ChunkSize {
            event,
            field: field_name,
            idx: field_index,
            expected_multiple_of: chunk_size,
            got: seq.len(),
        }));
    }

    let mut items = Vec::with_capacity(seq.len() / chunk_size);
    for (chunk_idx, chunk) in seq.chunks(chunk_size).enumerate() {
        let val = crate::normalize_value(Value::Seq(chunk.to_vec()));
        let item = T::deserialize(val).map_err(|e| {
            let mut err = DecodeError::custom(format!(
                "event {:?} field {:?}: failed to decode chunk item: {}",
                event, field_name, e
            ));
            err.prepend(PathSegment::Chunk(chunk_idx));
            err
        })?;
        items.push(item);
    }
    Ok(items)
}

fn is_zero_value(v: &Value) -> bool {
    matches!(
        v,
        Value::U8(0)
            | Value::U16(0)
            | Value::U32(0)
            | Value::U64(0)
            | Value::I8(0)
            | Value::I16(0)
            | Value::I32(0)
            | Value::I64(0)
    )
}

pub fn decode_chunks_zero_as_empty<T: DeserializeOwned>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
    chunk_size: usize,
    raw: Option<&Value>,
) -> Result<Vec<T>, DecodeError> {
    let value = raw.ok_or_else(|| {
        DecodeError::from(DecodeErrorKind::Missing {
            event,
            field: field_name,
            idx: field_index,
        })
    })?;

    if is_zero_value(value) {
        return Ok(Vec::new());
    }

    let seq = match value {
        Value::Seq(seq) => seq,
        other => {
            return Err(DecodeError::from(DecodeErrorKind::TypeMismatch {
                event,
                field: field_name,
                idx: field_index,
                expected: "seq or 0",
                got: crate::error::ValueKind::of(other),
            }));
        }
    };

    if seq.len() % chunk_size != 0 {
        return Err(DecodeError::from(DecodeErrorKind::ChunkSize {
            event,
            field: field_name,
            idx: field_index,
            expected_multiple_of: chunk_size,
            got: seq.len(),
        }));
    }

    let mut items = Vec::with_capacity(seq.len() / chunk_size);
    for (chunk_idx, chunk) in seq.chunks(chunk_size).enumerate() {
        let val = crate::normalize_value(Value::Seq(chunk.to_vec()));
        let item = T::deserialize(val).map_err(|e| {
            let mut err = DecodeError::custom(format!(
                "event {:?} field {:?}: failed to decode chunk item: {}",
                event, field_name, e
            ));
            err.prepend(PathSegment::Chunk(chunk_idx));
            err
        })?;
        items.push(item);
    }
    Ok(items)
}

#[derive(Clone, Copy, Default)]
pub struct CoerceFlags {
    pub num_to_bool: bool,
    pub bool_to_num: bool,
    pub str_num: bool,
}

fn coerce_alternatives(value: &Value, flags: CoerceFlags) -> Vec<Value> {
    let mut out = Vec::new();
    match value {
        Value::F32(f) => {
            out.extend([
                Value::F64(*f as f64),
                Value::I64(*f as i64),
                Value::U64(*f as u64),
                Value::I32(*f as i32),
                Value::I16(*f as i16),
                Value::U32(*f as u32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*f != 0.0));
            }
        }
        Value::F64(f) => {
            out.extend([
                Value::F32(*f as f32),
                Value::I64(*f as i64),
                Value::U64(*f as u64),
                Value::I32(*f as i32),
                Value::I16(*f as i16),
                Value::U32(*f as u32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*f != 0.0));
            }
        }
        Value::I8(i) => {
            out.extend([
                Value::I64(*i as i64),
                Value::I16(*i as i16),
                Value::I32(*i as i32),
                Value::U8(*i as u8),
                Value::U64(*i as u64),
                Value::F64(*i as f64),
                Value::F32(*i as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*i != 0));
            }
        }
        Value::I16(i) => {
            out.extend([
                Value::I64(*i as i64),
                Value::I32(*i as i32),
                Value::I8(*i as i8),
                Value::U16(*i as u16),
                Value::U64(*i as u64),
                Value::F64(*i as f64),
                Value::F32(*i as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*i != 0));
            }
        }
        Value::I32(i) => {
            out.extend([
                Value::I64(*i as i64),
                Value::I16(*i as i16),
                Value::I8(*i as i8),
                Value::U32(*i as u32),
                Value::U64(*i as u64),
                Value::F64(*i as f64),
                Value::F32(*i as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*i != 0));
            }
        }
        Value::I64(i) => {
            out.extend([
                Value::I32(*i as i32),
                Value::I16(*i as i16),
                Value::I8(*i as i8),
                Value::U64(*i as u64),
                Value::U32(*i as u32),
                Value::F64(*i as f64),
                Value::F32(*i as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*i != 0));
            }
        }
        Value::U8(u) => {
            out.extend([
                Value::U64(*u as u64),
                Value::U16(*u as u16),
                Value::U32(*u as u32),
                Value::I8(*u as i8),
                Value::I16(*u as i16),
                Value::I64(*u as i64),
                Value::F64(*u as f64),
                Value::F32(*u as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*u != 0));
            }
        }
        Value::U16(u) => {
            out.extend([
                Value::U64(*u as u64),
                Value::U32(*u as u32),
                Value::U8(*u as u8),
                Value::I16(*u as i16),
                Value::I32(*u as i32),
                Value::I64(*u as i64),
                Value::F64(*u as f64),
                Value::F32(*u as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*u != 0));
            }
        }
        Value::U32(u) => {
            out.extend([
                Value::U64(*u as u64),
                Value::U16(*u as u16),
                Value::U8(*u as u8),
                Value::I32(*u as i32),
                Value::I64(*u as i64),
                Value::F64(*u as f64),
                Value::F32(*u as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*u != 0));
            }
        }
        Value::U64(u) => {
            out.extend([
                Value::U32(*u as u32),
                Value::U16(*u as u16),
                Value::U8(*u as u8),
                Value::I64(*u as i64),
                Value::I32(*u as i32),
                Value::F64(*u as f64),
                Value::F32(*u as f32),
            ]);
            if flags.num_to_bool {
                out.push(Value::Bool(*u != 0));
            }
        }
        Value::Bool(b) if flags.bool_to_num => {
            let n: i64 = if *b { 1 } else { 0 };
            out.extend([
                Value::I64(n),
                Value::U64(n as u64),
                Value::I32(n as i32),
                Value::U8(n as u8),
            ]);
        }
        Value::String(s) if flags.str_num => {
            if let Ok(i) = s.parse::<i64>() {
                out.push(Value::I64(i));
            }
            if let Ok(u) = s.parse::<u64>() {
                out.push(Value::U64(u));
            }
            if let Ok(f) = s.parse::<f64>() {
                out.push(Value::F64(f));
            }
            match s.as_str() {
                "true" => out.push(Value::Bool(true)),
                "false" => out.push(Value::Bool(false)),
                _ => {}
            }
        }
        _ => {}
    }
    out
}

pub fn decode_field_coerce<T: DeserializeOwned>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
    flags: CoerceFlags,
    raw: Option<&Value>,
) -> Result<T, DecodeError> {
    let value = raw.ok_or_else(|| {
        DecodeError::from(DecodeErrorKind::Missing {
            event,
            field: field_name,
            idx: field_index,
        })
    })?;

    if let Ok(result) = crate::deserialize_value::<T>(value.clone()) {
        return Ok(result);
    }

    for alt in coerce_alternatives(value, flags) {
        if let Ok(result) = crate::deserialize_value::<T>(alt) {
            return Ok(result);
        }
    }

    crate::deserialize_value::<T>(value.clone()).map_err(|err| {
        DecodeError::custom(format!(
            "event {:?} field #{} {:?} (with coercion): {}",
            event, field_index, field_name, err
        ))
    })
}

fn apply_encode_coerce(value: Value, flags: CoerceFlags) -> Value {
    match &value {
        Value::Bool(b) if flags.bool_to_num => Value::I64(if *b { 1 } else { 0 }),
        Value::I8(i) if flags.num_to_bool => Value::Bool(*i != 0),
        Value::I16(i) if flags.num_to_bool => Value::Bool(*i != 0),
        Value::I32(i) if flags.num_to_bool => Value::Bool(*i != 0),
        Value::I64(i) if flags.num_to_bool => Value::Bool(*i != 0),
        Value::U8(u) if flags.num_to_bool => Value::Bool(*u != 0),
        Value::U16(u) if flags.num_to_bool => Value::Bool(*u != 0),
        Value::U32(u) if flags.num_to_bool => Value::Bool(*u != 0),
        Value::U64(u) if flags.num_to_bool => Value::Bool(*u != 0),
        Value::F32(f) if flags.num_to_bool => Value::Bool(*f != 0.0),
        Value::F64(f) if flags.num_to_bool => Value::Bool(*f != 0.0),
        _ => value,
    }
}

pub fn encode_field_coerce<T: Serialize>(value: &T, flags: CoerceFlags) -> anyhow::Result<Value> {
    let raw = serde_value::to_value(value)?;
    Ok(apply_encode_coerce(raw, flags))
}

pub fn decode_optional_field<T: DeserializeOwned>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
    raw: Option<&Value>,
) -> Result<Option<T>, DecodeError> {
    match raw {
        None => Ok(None),
        Some(Value::Unit) => Ok(None),
        Some(Value::Option(None)) => Ok(None),
        Some(value) => crate::deserialize_value(value.clone())
            .map(Some)
            .map_err(|err| {
                DecodeError::custom(format!(
                    "event {:?} field #{} {:?}: {}",
                    event, field_index, field_name, err
                ))
            }),
    }
}
