use anyhow::{anyhow, Result};
use rmp_serde::Serializer;
use serde::Serialize;
use serde_value::Value;
use std::io::Cursor;

pub fn unpack_frame(data: &[u8]) -> Result<(Vec<Value>, [u8; 2])> {
    if data.len() < 2 {
        return Err(anyhow!("frame too short to contain msgpack + trailer"));
    }
    let trailer: [u8; 2] = [data[data.len() - 2], data[data.len() - 1]];
    let body = &data[..data.len() - 2];
    let mut de = rmp_serde::Deserializer::new(Cursor::new(body));
    let values: Vec<Value> = serde::Deserialize::deserialize(&mut de)?;
    Ok((values, trailer))
}

pub fn repack_frame(values: &[Value], trailer: [u8; 2]) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    values.serialize(&mut Serializer::new(&mut buf))?;
    buf.extend_from_slice(&trailer);
    Ok(buf)
}
