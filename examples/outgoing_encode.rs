use anyhow::Result;
use packet_core::OutgoingPacket;
use packet_macros::{OutgoingFields, Packet};
use serde::Deserialize;

#[derive(Debug, Packet)]
#[packet(event = "po", allow_extra)]
pub struct Ping {}

#[derive(Debug, Packet)]
#[packet(event = "sb", allow_extra)]
pub struct SB {
    pub s1: String,
    pub s2: String,
}

#[derive(Debug, Deserialize, OutgoingFields)]
pub struct EnterGameLoadout {
    pub class_index: i64,
    pub spray_index: i64,
    pub hat_index: i64,
    pub body_index: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "en", allow_extra)]
pub struct EnterGame {
    #[packet(nested)]
    pub loadout: EnterGameLoadout,
    pub sixteen: i64,
    #[packet(coerce(bool_to_num))]
    pub has_scene_hook: bool,
}

#[derive(Debug, Packet)]
#[packet(event = "testPacket", allow_extra)]
pub struct TestFlag {
    #[packet(coerce(bool_to_num))]
    pub flag: bool,
}

fn main() -> Result<()> {
    let ping = Ping {};
    println!("po  -> {:?}", ping.to_values()?);

    let sb = SB {
        s1: "welc".into(),
        s2: "hello".into(),
    };
    println!("sb  -> {:?}", sb.to_values()?);

    let enter = EnterGame {
        loadout: EnterGameLoadout {
            class_index: 0,
            spray_index: 0,
            hat_index: -1,
            body_index: -1,
        },
        sixteen: 16,
        has_scene_hook: false,
    };
    println!("en  -> {:?}", enter.to_values()?);

    let test = TestFlag { flag: true };
    println!("tp  -> {:?}", test.to_values()?);

    Ok(())
}
