use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_value::Value;
use std::fmt;

#[derive(Clone, Default)]
pub struct NegOneOrDefault<T: Default>(pub T);

#[derive(Clone, Default)]
pub struct EmptyStringOrDefault<T: Default>(pub T);

#[derive(Clone, Default)]
pub struct EmptySeqOrDefault<T: Default>(pub T);

#[derive(Clone, Default)]
pub struct NullishOrDefault<T: Default>(pub T);

fn is_neg_one(v: &Value) -> bool {
    matches!(
        v,
        Value::I8(-1) | Value::I16(-1) | Value::I32(-1) | Value::I64(-1)
    )
}

fn is_empty_string(v: &Value) -> bool {
    matches!(v, Value::String(s) if s.is_empty())
}

fn is_empty_seq(v: &Value) -> bool {
    matches!(v, Value::Seq(s) if s.is_empty())
}

fn is_nullish(v: &Value) -> bool {
    matches!(v, Value::Unit | Value::Option(None) | Value::Bool(false))
}

macro_rules! impl_sentinel {
    ($ty:ident, $matcher:ident) => {
        impl<'de, T> Deserialize<'de> for $ty<T>
        where
            T: Default + DeserializeOwned,
        {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let value = Value::deserialize(d)?;
                if $matcher(&value) {
                    return Ok(Self(T::default()));
                }
                crate::deserialize_value::<T>(value)
                    .map(Self)
                    .map_err(serde::de::Error::custom)
            }
        }

        impl<T: Default + Serialize> Serialize for $ty<T> {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(s)
            }
        }

        impl<T: Default + fmt::Debug> fmt::Debug for $ty<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<T: Default> std::ops::Deref for $ty<T> {
            type Target = T;
            fn deref(&self) -> &T {
                &self.0
            }
        }

        impl<T: Default> std::ops::DerefMut for $ty<T> {
            fn deref_mut(&mut self) -> &mut T {
                &mut self.0
            }
        }

        impl<T: Default> From<T> for $ty<T> {
            fn from(t: T) -> Self {
                Self(t)
            }
        }
    };
}

impl_sentinel!(NegOneOrDefault, is_neg_one);
impl_sentinel!(EmptyStringOrDefault, is_empty_string);
impl_sentinel!(EmptySeqOrDefault, is_empty_seq);
impl_sentinel!(NullishOrDefault, is_nullish);
