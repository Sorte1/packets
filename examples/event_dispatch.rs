use anyhow::Result;
use packet_macros::{EventEnum, Packet};
use serde_value::Value;

#[derive(Debug, Packet)]
#[packet(event = "pi", allow_extra)]
pub struct Ping {}

#[derive(Debug, Packet)]
#[packet(event = "pir", scalar_as_seq, allow_extra)]
pub struct PingReturn {
    pub ping: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "3", scalar_as_seq, allow_extra)]
pub struct Kill {
    pub victim_sid: i64,
    pub killer_sid: i64,
}

#[derive(Debug, EventEnum)]
pub enum Event {
    Pi(Ping),
    Pir(PingReturn),
    Kill(Kill),

    #[event(unknown)]
    Unknown(String, Vec<Value>),
}

fn handle_event(event: Event) {
    match event {
        Event::Pi(_) => {} // todo!("send po"),
        Event::Pir(p) => println!("ping reply: {}ms", p.ping),
        Event::Kill(k) => println!("kill: {} -> {}", k.killer_sid, k.victim_sid),
        Event::Unknown(name, payload) => {
            println!("unknown event: {name} (payload len={})", payload.len())
        }
    }
}

fn main() -> Result<()> {
    let frames: Vec<(&str, Vec<Value>)> = vec![
        ("pi", vec![]),
        ("pir", vec![Value::I64(42)]),
        ("3", vec![Value::I64(2), Value::I64(1)]),
        ("zz", vec![Value::String("not a real event".into())]),
    ];

    for (name, payload) in frames {
        let event = Event::from_event_name(name, &payload)?;
        handle_event(event);
    }

    Ok(())
}
