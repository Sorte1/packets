use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use packet_core::error::ValueKind;
use packet_core::wire::unpack_frame;
use serde_value::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "packets", about = "Inspect Krunker packet frames")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Decode {
        #[arg(short, long, value_name = "PATH")]
        file: Option<PathBuf>,
        #[arg(long)]
        map: bool,
        #[arg(long)]
        explain: bool,
        input: Option<String>,
    },
    Drift {
        corpus: PathBuf,
    },
    Diff {
        a: String,
        b: String,
        #[arg(long)]
        files: bool,
    },
    Infer {
        corpus: PathBuf,
    },
}

fn main() -> Result<()> {
    let Cli { cmd } = Cli::parse();
    match cmd {
        Cmd::Decode {
            file,
            map,
            explain,
            input,
        } => decode(file, map, explain, input),
        Cmd::Drift { corpus } => drift(corpus),
        Cmd::Diff { a, b, files } => diff(a, b, files),
        Cmd::Infer { corpus } => infer(corpus),
    }
}

fn decode(file: Option<PathBuf>, map: bool, explain: bool, input: Option<String>) -> Result<()> {
    let bytes = read_bytes(file, input)?;
    let (values, trailer) = unpack_frame(&bytes)?;

    let event = match values.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(_) => "<not a string>",
        None => "<empty>",
    };

    println!("event:   {event}");
    println!("trailer: {:02x} {:02x}", trailer[0], trailer[1]);

    if explain {
        println!("payload: {} value(s)", values.len().saturating_sub(1));
        for (i, v) in values.iter().skip(1).enumerate() {
            let kind = ValueKind::of(v);
            let extra = match v {
                Value::Seq(s) => format!(" len={}", s.len()),
                Value::Map(m) => format!(" len={}", m.len()),
                Value::String(s) => format!(" len={}", s.len()),
                Value::Bytes(b) => format!(" len={}", b.len()),
                _ => String::new(),
            };
            println!(
                "  [{i}] <{kind}>{extra} {}",
                packet_core::fmt::pretty(v, 120)
            );
        }
        return Ok(());
    }

    if map {
        print!("{}", packet_core::fmt::snapshot_payload(&values[1..]));
        return Ok(());
    }

    println!("payload: {} value(s)", values.len().saturating_sub(1));
    for (i, v) in values.iter().skip(1).enumerate() {
        println!("  [{i}] {v:?}");
    }
    Ok(())
}

fn read_bytes(file: Option<PathBuf>, input: Option<String>) -> Result<Vec<u8>> {
    if let Some(path) = file {
        return std::fs::read(&path).with_context(|| format!("reading {}", path.display()));
    }
    let raw = match input {
        Some(s) => s,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };
    parse_hex(&raw)
}

#[derive(Default)]
struct EventSchema {
    frame_count: usize,
    arity_min: Option<usize>,
    arity_max: Option<usize>,
    field_kinds: Vec<BTreeSet<String>>,
}

fn drift(corpus: PathBuf) -> Result<()> {
    let bytes = std::fs::read(&corpus).with_context(|| format!("reading {}", corpus.display()))?;
    let frames = read_corpus(&bytes)?;
    let mut schemas: BTreeMap<String, EventSchema> = BTreeMap::new();
    let mut bad_frames = 0usize;

    for frame in &frames {
        match unpack_frame(frame) {
            Ok((values, _trailer)) => {
                let event_name = match values.first() {
                    Some(Value::String(s)) => s.clone(),
                    _ => "<no event name>".to_string(),
                };
                let payload = &values[1..];
                let entry = schemas.entry(event_name).or_default();
                entry.frame_count += 1;
                entry.arity_min = Some(
                    entry
                        .arity_min
                        .map_or(payload.len(), |m| m.min(payload.len())),
                );
                entry.arity_max = Some(
                    entry
                        .arity_max
                        .map_or(payload.len(), |m| m.max(payload.len())),
                );
                if entry.field_kinds.len() < payload.len() {
                    entry.field_kinds.resize_with(payload.len(), BTreeSet::new);
                }
                for (i, v) in payload.iter().enumerate() {
                    entry.field_kinds[i].insert(ValueKind::of(v).to_string());
                }
            }
            Err(_) => bad_frames += 1,
        }
    }

    println!("frames        : {}", frames.len());
    println!("bad frames    : {}", bad_frames);
    println!("distinct evts : {}", schemas.len());
    println!();

    for (name, s) in &schemas {
        let arity = match (s.arity_min, s.arity_max) {
            (Some(lo), Some(hi)) if lo == hi => format!("{lo}"),
            (Some(lo), Some(hi)) => format!("{lo}..{hi}  ⚠ variable"),
            _ => "?".to_string(),
        };
        println!("event {name:?} ({} frames, arity {arity})", s.frame_count);
        for (i, kinds) in s.field_kinds.iter().enumerate() {
            let joined: Vec<&str> = kinds.iter().map(String::as_str).collect();
            let marker = if joined.len() > 1 { "  ⚠ mixed" } else { "" };
            println!("  [{i}] {}{marker}", joined.join(" | "));
        }
        println!();
    }
    Ok(())
}

fn diff(a: String, b: String, files: bool) -> Result<()> {
    let bytes_a = if files {
        std::fs::read(&a).with_context(|| format!("reading {a}"))?
    } else {
        parse_hex(&a)?
    };
    let bytes_b = if files {
        std::fs::read(&b).with_context(|| format!("reading {b}"))?
    } else {
        parse_hex(&b)?
    };
    let (va, ta) = unpack_frame(&bytes_a)?;
    let (vb, tb) = unpack_frame(&bytes_b)?;
    let name_a = match va.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => "<none>",
    };
    let name_b = match vb.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => "<none>",
    };
    if name_a != name_b {
        println!("event   : {name_a:?}  vs  {name_b:?}  ⚠ different");
    } else {
        println!("event   : {name_a:?}");
    }
    if ta != tb {
        println!(
            "trailer : {:02x}{:02x}  vs  {:02x}{:02x}  ⚠",
            ta[0], ta[1], tb[0], tb[1]
        );
    } else {
        println!("trailer : {:02x}{:02x}", ta[0], ta[1]);
    }
    let pa = &va[1..];
    let pb = &vb[1..];
    let n = pa.len().max(pb.len());
    for i in 0..n {
        match (pa.get(i), pb.get(i)) {
            (Some(av), Some(bv)) if av == bv => {
                println!("  [{i}] = {}", packet_core::fmt::pretty(av, 80));
            }
            (Some(av), Some(bv)) => {
                println!(
                    "  [{i}] ≠ {}  →  {}",
                    packet_core::fmt::pretty(av, 80),
                    packet_core::fmt::pretty(bv, 80)
                );
            }
            (Some(av), None) => {
                println!("  [{i}] − {}", packet_core::fmt::pretty(av, 80));
            }
            (None, Some(bv)) => {
                println!("  [{i}] + {}", packet_core::fmt::pretty(bv, 80));
            }
            (None, None) => {}
        }
    }
    Ok(())
}

fn infer(corpus: PathBuf) -> Result<()> {
    let bytes = std::fs::read(&corpus).with_context(|| format!("reading {}", corpus.display()))?;
    let frames = read_corpus(&bytes)?;
    let mut schemas: BTreeMap<String, EventSchema> = BTreeMap::new();

    for frame in &frames {
        if let Ok((values, _)) = unpack_frame(frame) {
            let event_name = match values.first() {
                Some(Value::String(s)) => s.clone(),
                _ => continue,
            };
            let payload = &values[1..];
            let entry = schemas.entry(event_name).or_default();
            entry.frame_count += 1;
            entry.arity_min = Some(
                entry
                    .arity_min
                    .map_or(payload.len(), |m| m.min(payload.len())),
            );
            entry.arity_max = Some(
                entry
                    .arity_max
                    .map_or(payload.len(), |m| m.max(payload.len())),
            );
            if entry.field_kinds.len() < payload.len() {
                entry.field_kinds.resize_with(payload.len(), BTreeSet::new);
            }
            for (i, v) in payload.iter().enumerate() {
                entry.field_kinds[i].insert(ValueKind::of(v).to_string());
            }
        }
    }

    for (name, s) in &schemas {
        let struct_name = pascal_case(name);
        let attrs = if s.arity_min == s.arity_max {
            format!("#[packet(event = \"{}\")]", escape(name))
        } else {
            format!("#[packet(event = \"{}\", allow_extra)]", escape(name))
        };
        println!("#[derive(Debug, Packet)]");
        println!("{attrs}");
        println!("pub struct {struct_name} {{");
        let optional_threshold = s.arity_max.unwrap_or(0);
        for (i, kinds) in s.field_kinds.iter().enumerate() {
            let rust_ty = rust_type_for(kinds);
            let field_name = format!("field_{i}");
            let optional = i >= s.arity_min.unwrap_or(optional_threshold);
            let final_ty = if optional {
                format!("Option<{rust_ty}>")
            } else {
                rust_ty
            };
            println!("    pub {field_name}: {final_ty},");
        }
        println!("}}");
        println!();
    }

    Ok(())
}

fn pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize {
                out.extend(c.to_uppercase());
                capitalize = false;
            } else {
                out.push(c);
            }
        } else {
            capitalize = true;
        }
    }
    if out.is_empty() || !out.chars().next().unwrap().is_alphabetic() {
        format!("Event_{}", out)
    } else {
        out
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn rust_type_for(kinds: &BTreeSet<String>) -> String {
    if kinds.len() != 1 {
        return "serde_value::Value".to_string();
    }
    let only = kinds.iter().next().unwrap();
    match only.as_str() {
        "bool" => "bool".to_string(),
        "i8" => "i8".to_string(),
        "i16" => "i16".to_string(),
        "i32" => "i32".to_string(),
        "i64" => "i64".to_string(),
        "u8" => "u8".to_string(),
        "u16" => "u16".to_string(),
        "u32" => "u32".to_string(),
        "u64" => "u64".to_string(),
        "f32" => "f32".to_string(),
        "f64" => "f64".to_string(),
        "string" => "String".to_string(),
        "seq" => "Vec<serde_value::Value>".to_string(),
        "map" => "std::collections::BTreeMap<serde_value::Value, serde_value::Value>".to_string(),
        "bytes" => "Vec<u8>".to_string(),
        _ => "serde_value::Value".to_string(),
    }
}

fn read_corpus(bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut frames = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes.len() - i < 4 {
            return Err(anyhow!("truncated length prefix at offset {i}"));
        }
        let len = u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
        i += 4;
        if bytes.len() - i < len {
            return Err(anyhow!(
                "truncated frame: declared {len} bytes, only {} remain",
                bytes.len() - i
            ));
        }
        frames.push(bytes[i..i + len].to_vec());
        i += len;
    }
    Ok(frames)
}

fn parse_hex(input: &str) -> Result<Vec<u8>> {
    let mut cleaned = String::new();
    for token in input.split(|c: char| c.is_whitespace() || c == ',' || c == '[' || c == ']') {
        let t = token.trim_start_matches("0x").trim_start_matches("\\x");
        cleaned.push_str(t);
    }
    hex::decode(&cleaned).map_err(|e| anyhow!("bad hex: {e}"))
}
