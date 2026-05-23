use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_value::Value;
use std::fmt;

pub mod decode;
pub mod error;
pub mod wire;

pub use error::{DecodeError, DecodeErrorKind, ValueKind};

pub trait PacketMeta {
    const EVENT_NAME: &'static str;
}

pub trait PacketDecode: Sized {
    fn decode_payload(payload: &[Value]) -> Result<Self>;
}

pub trait OutgoingPacket {
    fn to_values(&self) -> Result<Vec<Value>>;
}

pub trait OutgoingFields {
    fn to_field_values(&self) -> Result<Vec<Value>>;
}

pub fn normalize_value(value: Value) -> Value {
    match value {
        Value::Unit => Value::Option(None),

        Value::Option(Some(inner)) => Value::Option(Some(Box::new(normalize_value(*inner)))),
        Value::Newtype(inner) => Value::Newtype(Box::new(normalize_value(*inner))),

        Value::Seq(seq) => Value::Seq(seq.into_iter().map(normalize_value).collect()),

        Value::Map(map) => Value::Map(
            map.into_iter()
                .map(|(k, v)| (normalize_value(k), normalize_value(v)))
                .collect(),
        ),

        other => other,
    }
}

pub fn payload_as_value(payload: &[Value]) -> Value {
    if payload.len() == 1 {
        payload[0].clone()
    } else {
        Value::Seq(payload.to_vec())
    }
}

pub fn deserialize_value<T: DeserializeOwned>(value: Value) -> Result<T> {
    let value = normalize_value(value);
    let out = T::deserialize(value)?;
    Ok(out)
}

pub fn deserialize_payload<T: DeserializeOwned>(payload: &[Value]) -> Result<T> {
    deserialize_value(payload_as_value(payload))
}

#[derive(Debug)]
pub struct PacketFieldError {
    pub event: &'static str,
    pub field_index: usize,
    pub field_name: &'static str,
    pub expected: &'static str,
    pub value: Option<serde_value::Value>,
    pub source: anyhow::Error,
}

impl std::fmt::Display for PacketFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to decode event {:?} field #{} {:?} as {}",
            self.event, self.field_index, self.field_name, self.expected
        )
    }
}

impl std::error::Error for PacketFieldError {}

#[derive(Clone, Default)]
pub struct ZeroOrDefault<T: Default>(pub T);

fn is_zero_integer(v: &Value) -> bool {
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

impl<'de, T> Deserialize<'de> for ZeroOrDefault<T>
where
    T: Default + DeserializeOwned,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if is_zero_integer(&value) {
            return Ok(Self(T::default()));
        }
        deserialize_value::<T>(value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

impl<T: Default + Serialize> Serialize for ZeroOrDefault<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<T: Default + fmt::Debug> fmt::Debug for ZeroOrDefault<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Default> std::ops::Deref for ZeroOrDefault<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Default> std::ops::DerefMut for ZeroOrDefault<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Default> From<T> for ZeroOrDefault<T> {
    fn from(val: T) -> Self {
        Self(val)
    }
}

pub trait OutgoingEventEnum: Sized {
    fn try_from_event(event_name: &str, payload: &[Value]) -> Option<Result<Self>>;

    fn passthrough(raw: Vec<u8>) -> Self;

    fn to_frame(&self, trailer: [u8; 2]) -> Result<Vec<u8>>;

    fn from_frame(data: &[u8]) -> Result<(Self, [u8; 2])> {
        let (values, trailer) = wire::unpack_frame(data)?;
        let event_name = match values.first() {
            Some(Value::String(s)) => s.clone(),
            Some(other) => return Err(anyhow!("expected event name string, got {:?}", other)),
            None => return Err(anyhow!("empty msgpack frame")),
        };
        let event = match Self::try_from_event(&event_name, &values[1..]) {
            Some(result) => result?,
            None => Self::passthrough(data.to_vec()),
        };
        Ok((event, trailer))
    }
}
