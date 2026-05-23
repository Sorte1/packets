use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_value::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OneOf2<A, B> {
    A(A),
    B(B),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OneOf3<A, B, C> {
    A(A),
    B(B),
    C(C),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OneOf4<A, B, C, D> {
    A(A),
    B(B),
    C(C),
    D(D),
}

fn try_decode<T: DeserializeOwned>(value: &Value) -> Option<T> {
    crate::deserialize_value::<T>(value.clone()).ok()
}

impl<'de, A, B> Deserialize<'de> for OneOf2<A, B>
where
    A: DeserializeOwned,
    B: DeserializeOwned,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        if let Some(a) = try_decode::<A>(&v) {
            return Ok(OneOf2::A(a));
        }
        if let Some(b) = try_decode::<B>(&v) {
            return Ok(OneOf2::B(b));
        }
        Err(serde::de::Error::custom(
            "OneOf2: no variant matched the value",
        ))
    }
}

impl<'de, A, B, C> Deserialize<'de> for OneOf3<A, B, C>
where
    A: DeserializeOwned,
    B: DeserializeOwned,
    C: DeserializeOwned,
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        if let Some(a) = try_decode::<A>(&v) {
            return Ok(OneOf3::A(a));
        }
        if let Some(b) = try_decode::<B>(&v) {
            return Ok(OneOf3::B(b));
        }
        if let Some(c) = try_decode::<C>(&v) {
            return Ok(OneOf3::C(c));
        }
        Err(serde::de::Error::custom(
            "OneOf3: no variant matched the value",
        ))
    }
}

impl<'de, A, B, C, D> Deserialize<'de> for OneOf4<A, B, C, D>
where
    A: DeserializeOwned,
    B: DeserializeOwned,
    C: DeserializeOwned,
    D: DeserializeOwned,
{
    fn deserialize<De: Deserializer<'de>>(de: De) -> Result<Self, De::Error> {
        let v = Value::deserialize(de)?;
        if let Some(a) = try_decode::<A>(&v) {
            return Ok(OneOf4::A(a));
        }
        if let Some(b) = try_decode::<B>(&v) {
            return Ok(OneOf4::B(b));
        }
        if let Some(c) = try_decode::<C>(&v) {
            return Ok(OneOf4::C(c));
        }
        if let Some(d) = try_decode::<D>(&v) {
            return Ok(OneOf4::D(d));
        }
        Err(serde::de::Error::custom(
            "OneOf4: no variant matched the value",
        ))
    }
}
