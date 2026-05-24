use packet_core::{OutgoingEventEnum, OutgoingPacket, PassthroughInfo, PassthroughReason};
use packet_macros::{OutgoingEventEnum as DeriveOutgoingEventEnum, Packet};

#[derive(Debug, Packet)]
#[packet(event = "ping", allow_extra)]
pub struct Ping {
    pub n: i64,
}

#[derive(Debug, DeriveOutgoingEventEnum)]
pub enum Outbound {
    Ping(Ping),
    #[event(unknown)]
    Unknown(PassthroughInfo),
}

fn ping_frame() -> Vec<u8> {
    let ping = Ping { n: 7 };
    let values = ping.to_values().unwrap();
    packet_core::wire::repack_frame(&values, [0xab, 0xcd]).unwrap()
}

#[test]
fn from_frame_returns_typed_event_for_known_name() {
    let frame = ping_frame();
    let (event, trailer) = Outbound::from_frame(&frame).unwrap();
    assert_eq!(trailer, [0xab, 0xcd]);
    assert!(matches!(event, Outbound::Ping(Ping { n: 7 })));
}

#[test]
fn from_frame_passthroughs_unknown_event_with_reason() {
    let frame = {
        let values = vec![
            serde_value::Value::String("zz".into()),
            serde_value::Value::I64(1),
        ];
        packet_core::wire::repack_frame(&values, [0, 0]).unwrap()
    };
    let (event, _) = Outbound::from_frame(&frame).unwrap();
    match event {
        Outbound::Unknown(info) => {
            assert_eq!(info.bytes, frame);
            assert!(matches!(info.reason, PassthroughReason::UnknownEvent(ref n) if n == "zz"));
        }
        other => panic!("expected passthrough, got {other:?}"),
    }
}

#[test]
fn from_frame_passthroughs_bad_frame_with_reason() {
    let frame: Vec<u8> = vec![0xff];
    let (event, _) = Outbound::from_frame(&frame).unwrap();
    assert!(matches!(
        event,
        Outbound::Unknown(PassthroughInfo {
            reason: PassthroughReason::BadFrame(_),
            ..
        })
    ));
}
