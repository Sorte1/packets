use std::cell::RefCell;

use crate::PacketDecode;
use serde_value::Value;

#[derive(Debug, Default, Clone)]
pub struct DecodeTrace {
    pub fields: Vec<FieldDecode>,
}

#[derive(Debug, Clone)]
pub struct FieldDecode {
    pub event: &'static str,
    pub field: &'static str,
    pub idx: usize,
    pub kind: FieldDecodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldDecodeKind {
    Direct,
    Coerced { alternatives_tried: usize },
    Default,
    OptionalNone,
    Chunks { count: usize },
    VarChunks { count: usize },
    Failed,
}

thread_local! {
    static TRACE_SINK: RefCell<Option<DecodeTrace>> = const { RefCell::new(None) };
}

pub fn decode_payload_traced<T: PacketDecode>(
    payload: &[Value],
) -> (anyhow::Result<T>, DecodeTrace) {
    TRACE_SINK.with(|sink| *sink.borrow_mut() = Some(DecodeTrace::default()));
    let result = T::decode_payload(payload);
    let trace =
        TRACE_SINK.with(|sink| sink.borrow_mut().take().unwrap_or_default());
    (result, trace)
}

pub fn record_field(
    event: &'static str,
    field: &'static str,
    idx: usize,
    kind: FieldDecodeKind,
) {
    #[cfg(feature = "tracing")]
    tracing::trace!(target: "packet_core::decode", event, field, idx, ?kind, "field decoded");

    TRACE_SINK.with(|sink| {
        if let Some(t) = sink.borrow_mut().as_mut() {
            t.fields.push(FieldDecode {
                event,
                field,
                idx,
                kind,
            });
        }
    });
}

impl DecodeTrace {
    pub fn summary(&self) -> String {
        use std::fmt::Write as _;
        let mut s = String::new();
        for f in &self.fields {
            let _ = writeln!(
                s,
                "{:?}.{} #{} -> {:?}",
                f.event, f.field, f.idx, f.kind
            );
        }
        s
    }
}
