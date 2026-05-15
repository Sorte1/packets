use anyhow::Result;
use packet_core::PacketDecode;
use packet_macros::Packet;
use serde::{Deserialize, Serialize};
use serde_value::Value;

#[derive(Debug, Packet)]
#[packet(event = "pir", scalar_as_seq, allow_extra)]
pub struct PingReturn {
    pub ping: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "start", scalar_as_seq, allow_extra)]
pub struct Start {
    pub timer: String,
    pub state: Option<i64>,
    pub spectating: bool,
    pub class_index: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerUpdateData {
    pub sid: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub x_d: f64,
    pub y_d: f64,
    pub step: f64,
    pub on_ground: i64,
    pub crouch: i64,
    pub weapon: i64,
    pub aim: i64,
    pub grapple: Option<f64>,
    pub ping: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "k", allow_extra)]
pub struct PlayerUpdate {
    #[packet(chunks(PlayerUpdateData, 13))]
    pub updates: Vec<PlayerUpdateData>,
    pub rate: i64,
    pub timestamp: i64,
}

fn main() -> Result<()> {
    let pir_payload = vec![Value::I64(42)];
    let pir = PingReturn::decode_payload(&pir_payload)?;
    println!("pir -> ping={}", pir.ping);

    let start_payload = vec![
        Value::String("0:30".into()),
        Value::Option(None),
        Value::Bool(false),
        Value::I64(3),
    ];
    let start = Start::decode_payload(&start_payload)?;
    println!(
        "start -> timer={:?} state={:?} class={}",
        start.timer, start.state, start.class_index
    );

    let one_player = |sid: i64, x: f64| {
        vec![
            Value::I64(sid),
            Value::F64(x),
            Value::F64(0.0),
            Value::F64(0.0),
            Value::F64(0.0),
            Value::F64(0.0),
            Value::F64(0.0),
            Value::I64(1),
            Value::I64(0),
            Value::I64(0),
            Value::I64(0),
            Value::Option(None),
            Value::I64(25),
        ]
    };
    let mut flat = one_player(1, 10.0);
    flat.extend(one_player(2, 20.0));
    let k_payload = vec![Value::Seq(flat), Value::I64(60), Value::I64(123_456)];

    let k = PlayerUpdate::decode_payload(&k_payload)?;
    println!(
        "k -> {} updates at rate={} ts={}",
        k.updates.len(),
        k.rate,
        k.timestamp
    );
    for u in &k.updates {
        println!("sid={} x={} ping={}", u.sid, u.x, u.ping);
    }

    Ok(())
}
