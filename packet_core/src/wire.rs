use anyhow::{anyhow, Result};
use rmp_serde::Serializer;
use serde::Serialize;
use serde_value::Value;
use std::io::Cursor;

pub fn unpack_frame_n<const N: usize>(data: &[u8]) -> Result<(Vec<Value>, [u8; N])> {
    if data.len() < N {
        return Err(anyhow!(
            "frame too short to contain msgpack + {}-byte trailer",
            N
        ));
    }
    let mut trailer = [0u8; N];
    let split = data.len() - N;
    trailer.copy_from_slice(&data[split..]);
    let body = &data[..split];
    let mut de = rmp_serde::Deserializer::new(Cursor::new(body));
    let values: Vec<Value> = serde::Deserialize::deserialize(&mut de)?;
    Ok((values, trailer))
}

pub fn repack_frame_n<const N: usize>(values: &[Value], trailer: [u8; N]) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    values.serialize(&mut Serializer::new(&mut buf))?;
    buf.extend_from_slice(&trailer);
    Ok(buf)
}

pub fn unpack_frame(data: &[u8]) -> Result<(Vec<Value>, [u8; 2])> {
    unpack_frame_n::<2>(data)
}

pub fn repack_frame(values: &[Value], trailer: [u8; 2]) -> Result<Vec<u8>> {
    repack_frame_n::<2>(values, trailer)
}
