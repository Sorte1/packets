use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_value::Value;

use crate::error::{DecodeError, DecodeErrorKind, PathSegment};
use crate::trace::{record_field, FieldDecodeKind};

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

    let out = crate::deserialize_value(value.clone()).map_err(|err| {
        DecodeError::custom(format!(
            "event {:?} field #{} {:?}: {}",
            event, field_index, field_name, err
        ))
    })?;
    record_field(event, field_name, field_index, FieldDecodeKind::Direct);
    Ok(out)
}

pub fn decode_field_with_default<T: DeserializeOwned + Default>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
    raw: Option<&Value>,
) -> Result<T, DecodeError> {
    match raw {
        Some(value) => {
            let out = crate::deserialize_value(value.clone()).map_err(|err| {
                DecodeError::custom(format!(
                    "event {:?} field #{} {:?}: {}",
                    event, field_index, field_name, err
                ))
            })?;
            record_field(event, field_name, field_index, FieldDecodeKind::Direct);
            Ok(out)
        }
        None => {
            record_field(event, field_name, field_index, FieldDecodeKind::Default);
            Ok(T::default())
        }
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
    record_field(
        event,
        field_name,
        field_index,
        FieldDecodeKind::Chunks { count: items.len() },
    );
    Ok(items)
}

pub fn decode_var_chunks<T: DeserializeOwned>(
    event: &'static str,
    field_index: usize,
    field_name: &'static str,
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

    let mut items: Vec<T> = Vec::new();
    let mut i = 0usize;
    let mut chunk_idx = 0usize;
    while i < seq.len() {
        let count = chunk_count_from(&seq[i]).ok_or_else(|| {
            let mut err = DecodeError::custom(format!(
                "event {:?} field {:?}: expected numeric chunk length at position {}, got {:?}",
                event, field_name, i, &seq[i]
            ));
            err.prepend(PathSegment::Chunk(chunk_idx));
            err
        })?;
        i += 1;
        if i + count > seq.len() {
            let mut err = DecodeError::custom(format!(
                "event {:?} field {:?}: chunk declares {} values but only {} remain",
                event,
                field_name,
                count,
                seq.len() - i
            ));
            err.prepend(PathSegment::Chunk(chunk_idx));
            return Err(err);
        }
        let chunk_values = seq[i..i + count].to_vec();
        let val = crate::normalize_value(Value::Seq(chunk_values));
        let item = T::deserialize(val).map_err(|e| {
            let mut err = DecodeError::custom(format!(
                "event {:?} field {:?}: failed to decode var chunk: {}",
                event, field_name, e
            ));
            err.prepend(PathSegment::Chunk(chunk_idx));
            err
        })?;
        items.push(item);
        i += count;
        chunk_idx += 1;
    }
    record_field(
        event,
        field_name,
        field_index,
        FieldDecodeKind::VarChunks { count: items.len() },
    );
    Ok(items)
}

fn chunk_count_from(v: &Value) -> Option<usize> {
    match v {
        Value::U8(n) => Some(*n as usize),
        Value::U16(n) => Some(*n as usize),
        Value::U32(n) => Some(*n as usize),
        Value::U64(n) => Some(*n as usize),
        Value::I8(n) if *n >= 0 => Some(*n as usize),
        Value::I16(n) if *n >= 0 => Some(*n as usize),
        Value::I32(n) if *n >= 0 => Some(*n as usize),
        Value::I64(n) if *n >= 0 => Some(*n as usize),
        _ => None,
    }
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
    record_field(
        event,
        field_name,
        field_index,
        FieldDecodeKind::Chunks { count: items.len() },
    );
    Ok(items)
}

#[derive(Clone, Copy, Default)]
pub struct CoerceFlags {
    pub num_to_bool: bool,
    pub bool_to_num: bool,
    pub str_num: bool,
    pub lossless: bool,
}

#[inline]
fn lossless_int_to_f64(src: i128) -> Option<f64> {
    if src.unsigned_abs() <= (1u128 << 53) {
        Some(src as f64)
    } else {
        None
    }
}

#[inline]
fn lossless_int_to_f32(src: i128) -> Option<f32> {
    if src.unsigned_abs() <= (1u128 << 24) {
        Some(src as f32)
    } else {
        None
    }
}

#[inline]
fn lossless_f64_to_int<T>(src: f64, lossless: bool) -> Option<T>
where
    T: Copy + TryFrom<i128>,
{
    if !lossless {
        return None;
    }
    if !src.is_finite() || src.fract() != 0.0 {
        return None;
    }
    let as_i128 = src as i128;
    T::try_from(as_i128).ok()
}

fn push_signed_int_alternatives(out: &mut Vec<Value>, src: i128, lossless: bool) {
    if let Ok(v) = i8::try_from(src) {
        out.push(Value::I8(v));
    }
    if let Ok(v) = i16::try_from(src) {
        out.push(Value::I16(v));
    }
    if let Ok(v) = i32::try_from(src) {
        out.push(Value::I32(v));
    }
    if let Ok(v) = i64::try_from(src) {
        out.push(Value::I64(v));
    }
    if let Ok(v) = u8::try_from(src) {
        out.push(Value::U8(v));
    }
    if let Ok(v) = u16::try_from(src) {
        out.push(Value::U16(v));
    }
    if let Ok(v) = u32::try_from(src) {
        out.push(Value::U32(v));
    }
    if let Ok(v) = u64::try_from(src) {
        out.push(Value::U64(v));
    }
    push_int_float_alternatives(out, src, lossless);
}

fn push_unsigned_int_alternatives(out: &mut Vec<Value>, src: u128, lossless: bool) {
    if let Ok(v) = u8::try_from(src) {
        out.push(Value::U8(v));
    }
    if let Ok(v) = u16::try_from(src) {
        out.push(Value::U16(v));
    }
    if let Ok(v) = u32::try_from(src) {
        out.push(Value::U32(v));
    }
    if let Ok(v) = u64::try_from(src) {
        out.push(Value::U64(v));
    }
    if let Ok(v) = i8::try_from(src) {
        out.push(Value::I8(v));
    }
    if let Ok(v) = i16::try_from(src) {
        out.push(Value::I16(v));
    }
    if let Ok(v) = i32::try_from(src) {
        out.push(Value::I32(v));
    }
    if let Ok(v) = i64::try_from(src) {
        out.push(Value::I64(v));
    }
    if let Ok(signed) = i128::try_from(src) {
        push_int_float_alternatives(out, signed, lossless);
    }
}

fn push_int_float_alternatives(out: &mut Vec<Value>, src: i128, lossless: bool) {
    if lossless {
        if let Some(f) = lossless_int_to_f64(src) {
            out.push(Value::F64(f));
        }
        if let Some(f) = lossless_int_to_f32(src) {
            out.push(Value::F32(f));
        }
    } else {
        out.push(Value::F64(src as f64));
        out.push(Value::F32(src as f32));
    }
}

fn push_float_alternatives(out: &mut Vec<Value>, src: f64, lossless: bool) {
    if lossless {
        let as_f32 = src as f32;
        if as_f32 as f64 == src || (src.is_nan() && as_f32.is_nan()) {
            out.push(Value::F32(as_f32));
        }
        if let Some(v) = lossless_f64_to_int::<i64>(src, true) {
            out.push(Value::I64(v));
        }
        if let Some(v) = lossless_f64_to_int::<i32>(src, true) {
            out.push(Value::I32(v));
        }
        if let Some(v) = lossless_f64_to_int::<i16>(src, true) {
            out.push(Value::I16(v));
        }
        if let Some(v) = lossless_f64_to_int::<i8>(src, true) {
            out.push(Value::I8(v));
        }
        if let Some(v) = lossless_f64_to_int::<u64>(src, true) {
            out.push(Value::U64(v));
        }
        if let Some(v) = lossless_f64_to_int::<u32>(src, true) {
            out.push(Value::U32(v));
        }
        if let Some(v) = lossless_f64_to_int::<u16>(src, true) {
            out.push(Value::U16(v));
        }
        if let Some(v) = lossless_f64_to_int::<u8>(src, true) {
            out.push(Value::U8(v));
        }
    } else {
        out.extend([
            Value::F32(src as f32),
            Value::I64(src as i64),
            Value::U64(src as u64),
            Value::I32(src as i32),
            Value::I16(src as i16),
            Value::U32(src as u32),
        ]);
    }
}

fn coerce_alternatives(value: &Value, flags: CoerceFlags) -> Vec<Value> {
    let mut out = Vec::new();
    let lossless = flags.lossless;
    match value {
        Value::F32(f) => push_float_alternatives(&mut out, *f as f64, lossless),
        Value::F64(f) => push_float_alternatives(&mut out, *f, lossless),
        Value::I8(i) => push_signed_int_alternatives(&mut out, *i as i128, lossless),
        Value::I16(i) => push_signed_int_alternatives(&mut out, *i as i128, lossless),
        Value::I32(i) => push_signed_int_alternatives(&mut out, *i as i128, lossless),
        Value::I64(i) => push_signed_int_alternatives(&mut out, *i as i128, lossless),
        Value::U8(u) => push_unsigned_int_alternatives(&mut out, *u as u128, lossless),
        Value::U16(u) => push_unsigned_int_alternatives(&mut out, *u as u128, lossless),
        Value::U32(u) => push_unsigned_int_alternatives(&mut out, *u as u128, lossless),
        Value::U64(u) => push_unsigned_int_alternatives(&mut out, *u as u128, lossless),
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
    if flags.num_to_bool {
        if let Some(b) = value_as_bool(value) {
            out.push(Value::Bool(b));
        }
    }
    out
}

fn value_as_bool(v: &Value) -> Option<bool> {
    match v {
        Value::I8(i) => Some(*i != 0),
        Value::I16(i) => Some(*i != 0),
        Value::I32(i) => Some(*i != 0),
        Value::I64(i) => Some(*i != 0),
        Value::U8(u) => Some(*u != 0),
        Value::U16(u) => Some(*u != 0),
        Value::U32(u) => Some(*u != 0),
        Value::U64(u) => Some(*u != 0),
        Value::F32(f) => Some(*f != 0.0),
        Value::F64(f) => Some(*f != 0.0),
        _ => None,
    }
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
        record_field(
            event,
            field_name,
            field_index,
            FieldDecodeKind::Coerced {
                alternatives_tried: 0,
            },
        );
        return Ok(result);
    }

    let alternatives = coerce_alternatives(value, flags);
    for (i, alt) in alternatives.iter().enumerate() {
        if let Ok(result) = crate::deserialize_value::<T>(alt.clone()) {
            record_field(
                event,
                field_name,
                field_index,
                FieldDecodeKind::Coerced {
                    alternatives_tried: i + 1,
                },
            );
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
        None | Some(Value::Unit) | Some(Value::Option(None)) => {
            record_field(
                event,
                field_name,
                field_index,
                FieldDecodeKind::OptionalNone,
            );
            Ok(None)
        }
        Some(value) => {
            let out = crate::deserialize_value(value.clone())
                .map(Some)
                .map_err(|err| {
                    DecodeError::custom(format!(
                        "event {:?} field #{} {:?}: {}",
                        event, field_index, field_name, err
                    ))
                })?;
            record_field(event, field_name, field_index, FieldDecodeKind::Direct);
            Ok(out)
        }
    }
}
