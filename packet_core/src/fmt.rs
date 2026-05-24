use serde_value::Value;
use std::collections::BTreeMap;
use std::fmt::Write;

const TRUNCATE_TAIL: &str = "…+";

pub fn pretty(value: &Value, max_len: usize) -> String {
    let mut out = String::new();
    write_value(&mut out, value, max_len);
    out
}

pub fn snapshot_payload(payload: &[Value]) -> String {
    let mut out = String::new();
    for (i, v) in payload.iter().enumerate() {
        out.push_str(&format!("[{i}] {}\n", pretty(v, usize::MAX)));
    }
    out
}

fn write_value(out: &mut String, value: &Value, budget: usize) {
    match value {
        Value::Bool(b) => write!(out, "{b}").unwrap(),
        Value::I8(n) => write!(out, "{n}i8").unwrap(),
        Value::I16(n) => write!(out, "{n}i16").unwrap(),
        Value::I32(n) => write!(out, "{n}i32").unwrap(),
        Value::I64(n) => write!(out, "{n}i64").unwrap(),
        Value::U8(n) => write!(out, "{n}u8").unwrap(),
        Value::U16(n) => write!(out, "{n}u16").unwrap(),
        Value::U32(n) => write!(out, "{n}u32").unwrap(),
        Value::U64(n) => write!(out, "{n}u64").unwrap(),
        Value::F32(n) => write!(out, "{n}f32").unwrap(),
        Value::F64(n) => write!(out, "{n}f64").unwrap(),
        Value::Char(c) => write!(out, "'{}'", c.escape_default()).unwrap(),
        Value::String(s) => {
            out.push('"');
            for c in s.chars() {
                for esc in c.escape_default() {
                    out.push(esc);
                }
            }
            out.push('"');
        }
        Value::Bytes(b) => write!(out, "<{} bytes>", b.len()).unwrap(),
        Value::Unit => out.push_str("()"),
        Value::Option(None) => out.push_str("None"),
        Value::Option(Some(inner)) => {
            out.push_str("Some(");
            write_value(out, inner, budget.saturating_sub(out.len()));
            out.push(')');
        }
        Value::Newtype(inner) => write_value(out, inner, budget),
        Value::Seq(items) => write_seq(out, items, budget),
        Value::Map(entries) => write_map(out, entries, budget),
    }
}

fn write_seq(out: &mut String, items: &[Value], budget: usize) {
    out.push('[');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let before = out.len();
        write_value(out, item, budget.saturating_sub(out.len()));
        if out.len() > budget {
            out.truncate(before);
            let remaining = items.len() - i;
            write!(out, "{TRUNCATE_TAIL}{remaining} more").unwrap();
            out.push(']');
            return;
        }
    }
    out.push(']');
}

fn write_map(out: &mut String, entries: &BTreeMap<Value, Value>, budget: usize) {
    let mut sorted: Vec<(&Value, &Value)> = entries.iter().collect();
    sorted.sort_by_key(|(k, _)| pretty(k, 80));

    out.push('{');
    for (i, (k, v)) in sorted.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let before = out.len();
        write_value(out, k, budget.saturating_sub(out.len()));
        out.push_str(": ");
        write_value(out, v, budget.saturating_sub(out.len()));
        if out.len() > budget {
            out.truncate(before);
            let remaining = sorted.len() - i;
            write!(out, "{TRUNCATE_TAIL}{remaining} more").unwrap();
            out.push('}');
            return;
        }
    }
    out.push('}');
}
