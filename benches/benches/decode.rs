use criterion::{black_box, criterion_group, criterion_main, Criterion};
use packet_core::{OutgoingPacket, PacketDecode};
use packet_macros::Packet;
use serde::{Deserialize, Serialize};
use serde_value::Value;

#[derive(Debug, Packet)]
#[packet(event = "start", scalar_as_seq, allow_extra)]
struct Start {
    timer: String,
    state: Option<i64>,
    spectating: bool,
    class_index: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerUpdateData {
    sid: i64,
    x: f64,
    y: f64,
    z: f64,
    x_d: f64,
    y_d: f64,
    step: f64,
    on_ground: i64,
    crouch: i64,
    weapon: i64,
    aim: i64,
    grapple: Option<f64>,
    ping: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "k", allow_extra)]
struct PlayerUpdate {
    #[packet(chunks(PlayerUpdateData, 13))]
    updates: Vec<PlayerUpdateData>,
    rate: i64,
    timestamp: i64,
}

fn start_payload() -> Vec<Value> {
    vec![
        Value::String("0:30".into()),
        Value::Option(None),
        Value::Bool(false),
        Value::I64(3),
    ]
}

fn player_update_payload(n: usize) -> Vec<Value> {
    let mut flat = Vec::with_capacity(n * 13);
    for sid in 0..n as i64 {
        flat.extend([
            Value::I64(sid),
            Value::F64(sid as f64 * 1.5),
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
        ]);
    }
    vec![Value::Seq(flat), Value::I64(60), Value::I64(123_456)]
}

fn bench_decode_start(c: &mut Criterion) {
    let payload = start_payload();
    c.bench_function("decode_start", |b| {
        b.iter(|| Start::decode_payload(black_box(&payload)).unwrap());
    });
}

fn bench_decode_player_update(c: &mut Criterion) {
    let payload = player_update_payload(50);
    c.bench_function("decode_player_update_50", |b| {
        b.iter(|| PlayerUpdate::decode_payload(black_box(&payload)).unwrap());
    });
}

fn bench_roundtrip_outgoing(c: &mut Criterion) {
    let payload = start_payload();
    let start = Start::decode_payload(&payload).unwrap();
    c.bench_function("roundtrip_outgoing_start", |b| {
        b.iter(|| {
            let values = start.to_values().unwrap();
            Start::decode_payload(black_box(&values[1..])).unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_decode_start,
    bench_decode_player_update,
    bench_roundtrip_outgoing
);
criterion_main!(benches);
