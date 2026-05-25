use crate::map::types::Modes;
use anyhow::Result;
use packet_core::{OutgoingPacket, PacketDecode, PacketMeta};
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize};
use serde_value::Value;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Leaderboard {
    pub sid: i64,
    pub score: Option<i64>,
    pub kills: Option<i64>,
    pub deaths: Option<i64>,
    pub caps: Option<i64>,
    pub assists: Option<i64>,
    pub confirms: Option<i64>,
    pub denies: Option<i64>,
    pub objective: Option<i64>,
    pub deposits: Option<i64>,
    pub pickups: Option<i64>,
    pub points: Option<i64>,
    pub zombies: Option<i64>,
    pub downs: Option<i64>,
    pub dmg: Option<i64>,
    pub weapon_tier: Option<i64>,
    pub raw_stats: Vec<i64>,
    pub extras: Vec<i64>,
}

#[derive(Debug, Clone, Copy)]
enum Column {
    Score,
    Kills,
    Deaths,
    Caps,
    Assists,
    Confirms,
    Denies,
    Objective,
    Deposits,
    Pickups,
    Points,
    Zombies,
    Downs,
    Dmg,
    WeaponTier,
}

impl Leaderboard {
    fn set(&mut self, col: Column, value: i64) {
        match col {
            Column::Score => self.score = Some(value),
            Column::Kills => self.kills = Some(value),
            Column::Deaths => self.deaths = Some(value),
            Column::Caps => self.caps = Some(value),
            Column::Assists => self.assists = Some(value),
            Column::Confirms => self.confirms = Some(value),
            Column::Denies => self.denies = Some(value),
            Column::Objective => self.objective = Some(value),
            Column::Deposits => self.deposits = Some(value),
            Column::Pickups => self.pickups = Some(value),
            Column::Points => self.points = Some(value),
            Column::Zombies => self.zombies = Some(value),
            Column::Downs => self.downs = Some(value),
            Column::Dmg => self.dmg = Some(value),
            Column::WeaponTier => self.weapon_tier = Some(value),
        }
    }
}

const DEFAULT_COLS: &[Column] = &[Column::Score, Column::Kills, Column::Deaths];
const CTF_COLS: &[Column] = &[Column::Caps, Column::Score];
const WIDE_COLS: &[Column] = &[
    Column::Score,
    Column::Kills,
    Column::Deaths,
    Column::Assists,
];
const SCORE_ONLY_COLS: &[Column] = &[Column::Score];

fn columns_for_mode(mode: Modes) -> &'static [Column] {
    use Column::*;
    match mode {
        Modes::ffa | Modes::tdm | Modes::krank | Modes::clas | Modes::shrp | Modes::bhffa => {
            &[Score, Kills, Deaths]
        }
        Modes::ctf => &[Caps, Score],
        Modes::kc => &[Confirms, Score, Denies],
        Modes::tdf | Modes::car => &[Objective, Score],
        Modes::chs | Modes::imp => &[Score, Kills],
        Modes::point => &[Score, Objective, Kills],
        Modes::depoffa => &[Deposits, Kills, Score],
        Modes::depo => &[Deposits, Kills, Denies, Score],
        Modes::dom => &[Score, Caps],
        Modes::aon => &[Score, Kills],
        Modes::md => &[Score, Kills, Pickups],
        Modes::zom => &[Points, Zombies, Downs, Dmg],
        Modes::gun => &[WeaponTier, Kills],
        _ => DEFAULT_COLS,
    }
}

fn column_names_for_mode(mode: Modes) -> &'static [&'static str] {
    match mode {
        Modes::ffa | Modes::tdm | Modes::krank | Modes::clas | Modes::shrp | Modes::bhffa => {
            &["score", "kills", "deaths"]
        }
        Modes::ctf => &["caps", "score"],
        Modes::kc => &["confirms", "score", "denies"],
        Modes::tdf | Modes::car => &["objective", "score"],
        Modes::chs | Modes::imp => &["score", "kills"],
        Modes::point => &["score", "objective", "kills"],
        Modes::depoffa => &["deposits", "kills", "score"],
        Modes::depo => &["deposits", "kills", "denies", "score"],
        Modes::dom => &["score", "caps"],
        Modes::aon => &["score", "kills"],
        Modes::md => &["score", "kills", "pickups"],
        Modes::zom => &["points", "zombies", "downs", "dmg"],
        Modes::gun => &["weaponTier", "kills"],
        _ => &["score", "kills", "deaths"],
    }
}

fn normalize_mode_name(mode: &str) -> String {
    let mut out = String::with_capacity(mode.len());
    let mut wrote_sep = false;
    for ch in mode.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            wrote_sep = false;
        } else if !wrote_sep {
            out.push('_');
            wrote_sep = true;
        }
    }
    out.trim_matches('_').to_string()
}

fn columns_for_mode_name(mode: &str) -> &'static [Column] {
    use Column::*;
    match normalize_mode_name(mode).as_str() {
        "ffa" | "free_for_all" | "tdm" | "team_deathmatch" | "krank" | "kranked_ffa" | "clas"
        | "classic_ffa" | "shrp" | "sharp_shooter" | "bhffa" | "bighead_ffa" => {
            &[Score, Kills, Deaths]
        }
        "ctf" | "capture_the_flag" => &[Caps, Score],
        "kc" | "kill_confirmed" => &[Confirms, Score, Denies],
        "tdf" | "team_defender" | "car" | "carrier" => &[Objective, Score],
        "chs" | "chaos_snipers" | "imp" | "impulsed" => &[Score, Kills],
        "point" | "hardpoint" => &[Score, Objective, Kills],
        "depoffa" | "deposit_ffa" => &[Deposits, Kills, Score],
        "depo" | "deposit" => &[Deposits, Kills, Denies, Score],
        "dom" | "domination" => &[Score, Caps],
        "aon" | "all_or_nothing" => &[Score, Kills],
        "md" | "mag_dump" => &[Score, Kills, Pickups],
        "zom" | "zombies" => &[Points, Zombies, Downs, Dmg],
        "gun" | "gun_game" => &[WeaponTier, Kills],
        _ => DEFAULT_COLS,
    }
}

pub fn column_names_for_mode_index(mode_index: usize) -> &'static [&'static str] {
    column_names_for_mode(Modes::from_index(mode_index))
}

fn decode_entry<E>(chunk: &[Value], columns: &[Column]) -> Result<Leaderboard, E>
where
    E: serde::de::Error,
{
    if chunk.is_empty() {
        return Err(E::custom("leaderboard entry chunk is empty"));
    }
    let sid = i64::deserialize(chunk[0].clone().into_deserializer()).map_err(E::custom)?;
    let mut entry = Leaderboard {
        sid,
        ..Default::default()
    };
    for (i, val) in chunk.iter().skip(1).enumerate() {
        let n = i64::deserialize(val.clone().into_deserializer()).map_err(E::custom)?;
        entry.raw_stats.push(n);
        if let Some(&col) = columns.get(i) {
            entry.set(col, n);
        } else {
            entry.extras.push(n);
        }
    }
    Ok(entry)
}

fn decode_with_columns<E>(seq: Vec<Value>, columns: &[Column]) -> Result<Vec<Leaderboard>, E>
where
    E: serde::de::Error,
{
    let chunk_size = columns.len() + 1;
    if seq.len() % chunk_size != 0 {
        return Err(E::custom(format!(
            "expected a multiple of {} leaderboard elements, got {}",
            chunk_size,
            seq.len()
        )));
    }
    seq.chunks(chunk_size)
        .map(|c| decode_entry::<E>(c, columns))
        .collect()
}

pub fn deserialize_leader<'de, D>(deserializer: D) -> Result<Vec<Leaderboard>, D::Error>
where
    D: Deserializer<'de>,
{
    let seq: Vec<Value> = Vec::deserialize(deserializer)?;

    for cols in [DEFAULT_COLS, CTF_COLS, WIDE_COLS, SCORE_ONLY_COLS] {
        if seq.len() % (cols.len() + 1) == 0 {
            return decode_with_columns::<D::Error>(seq, cols);
        }
    }

    Err(serde::de::Error::custom(format!(
        "unsupported leaderboard payload width: {} elements",
        seq.len()
    )))
}

#[derive(Serialize, Debug)]
pub struct LeaderboardPacket {
    pub entries: Vec<Leaderboard>,
}

impl<'de> Deserialize<'de> for LeaderboardPacket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            entries: deserialize_leader(deserializer)?,
        })
    }
}

impl packet_core::PacketMeta for LeaderboardPacket {
    const EVENT_NAME: &'static str = "7";
}

impl PacketDecode for LeaderboardPacket {
    fn decode_payload(payload: &[Value]) -> Result<Self> {
        packet_core::deserialize_payload(payload)
    }
}

impl OutgoingPacket for LeaderboardPacket {
    fn to_values(&self) -> Result<Vec<Value>> {
        let mut seq = Vec::new();
        for e in &self.entries {
            seq.push(Value::I64(e.sid));
            for &n in &e.raw_stats {
                seq.push(Value::I64(n));
            }
        }
        Ok(vec![
            Value::String(Self::EVENT_NAME.to_string()),
            Value::Seq(seq),
        ])
    }
}
