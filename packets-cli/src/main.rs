use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use packet_core::wire::unpack_frame;
use serde_value::Value;
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
        input: Option<String>,
    },
}

fn main() -> Result<()> {
    let Cli { cmd } = Cli::parse();
    match cmd {
        Cmd::Decode { file, input } => decode(file, input),
    }
}

fn decode(file: Option<PathBuf>, input: Option<String>) -> Result<()> {
    let bytes = read_bytes(file, input)?;
    let (values, trailer) = unpack_frame(&bytes)?;

    let event = match values.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(_) => "<not a string>",
        None => "<empty>",
    };

    println!("event:   {event}");
    println!("trailer: {:02x} {:02x}", trailer[0], trailer[1]);
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

fn parse_hex(input: &str) -> Result<Vec<u8>> {
    let mut cleaned = String::new();
    for token in input.split(|c: char| c.is_whitespace() || c == ',' || c == '[' || c == ']') {
        let t = token.trim_start_matches("0x").trim_start_matches("\\x");
        cleaned.push_str(t);
    }
    hex::decode(&cleaned).map_err(|e| anyhow!("bad hex: {e}"))
}
