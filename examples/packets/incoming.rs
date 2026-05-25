use anyhow::Result;
use packet_core::OutgoingPacket;
use packet_macros::{EventEnum, Packet};
use serde::{Deserialize, Deserializer, Serialize};
use serde_value::Value;

pub use super::leaderboard::{
    column_names_for_mode_index, deserialize_leader, Leaderboard, LeaderboardPacket,
};

#[derive(Debug, Packet)]
#[packet(event = "pir", scalar_as_seq, allow_extra)]
pub struct PingReturn {
    pub ping: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "t", scalar_as_seq, allow_extra)]
pub struct Time {
    pub time_left: String,
    // 0 normal, 1 timer.end, 2 matchover, 3 matchabandoned
    pub timer_context: Option<i32>,
    pub game_timer: Option<i32>,
}

#[derive(Debug, Packet)]
#[packet(event = "pi", allow_extra)]
pub struct Ping {}

#[derive(Debug, Packet)]
#[packet(event = "ready", scalar_as_seq, allow_extra)]
pub struct Ready {
    pub value: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "remail", allow_extra)]
pub struct ReMail {}

#[derive(Debug, Packet)]
#[packet(event = "inst-id", scalar_as_seq, allow_extra)]
pub struct InstID {
    pub game_id: String,
}

#[derive(Debug, Packet)]
#[packet(event = "io-init", scalar_as_seq, allow_extra)]
pub struct IoInit {
    pub socket_id: String,
}

#[derive(Debug, Packet)]
#[packet(event = "2", scalar_as_seq, allow_extra)]
pub struct RemovePlayer {
    pub sid: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "cntry", scalar_as_seq, allow_extra)]
pub struct Country {
    pub country: String,
}

#[derive(Debug, Packet)]
#[packet(event = "load", scalar_as_seq, allow_extra)]
pub struct Load {
    pub id: i64,
    pub uuid: String,
}

#[derive(Debug, Packet)]
#[packet(event = "uid", scalar_as_seq, allow_extra)]
pub struct UID {
    pub uid: String,
}

#[derive(Debug, Packet)]
#[packet(event = "error", scalar_as_seq, allow_extra)]
pub struct Error {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Challenge {
    pub algorithm: String,
    pub challenge: String,
    #[serde(rename = "maxnumber")]
    pub max_number: u64,
    pub salt: String,
    pub signature: String,
}

#[derive(Debug, Packet)]
#[packet(event = "_0", scalar_as_seq, allow_extra)]
pub struct Captcha {
    pub a: i64,
    pub challenge: Challenge,
}

fn deserialize_f64_lenient<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let val = Value::deserialize(deserializer)?;
    match val {
        Value::F64(f) => Ok(f),
        Value::F32(f) => Ok(f as f64),
        Value::I64(i) => Ok(i as f64),
        Value::I32(i) => Ok(i as f64),
        Value::I16(i) => Ok(i as f64),
        Value::I8(i) => Ok(i as f64),
        Value::U64(u) => Ok(u as f64),
        Value::U32(u) => Ok(u as f64),
        Value::U16(u) => Ok(u as f64),
        Value::U8(u) => Ok(u as f64),
        Value::String(s) => s.parse::<f64>().map_err(D::Error::custom),
        other => Err(D::Error::custom(format!(
            "expected number or numeric string, got {:?}",
            other
        ))),
    }
}

fn deserialize_grapple<'de, D>(deserializer: D) -> Result<Option<(f64, f64, f64)>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Value::deserialize(deserializer)?;
    match val {
        Value::Seq(seq) if seq.len() == 3 => {
            let x = f64::deserialize(seq[0].clone()).map_err(serde::de::Error::custom)?;
            let y = f64::deserialize(seq[1].clone()).map_err(serde::de::Error::custom)?;
            let z = f64::deserialize(seq[2].clone()).map_err(serde::de::Error::custom)?;
            Ok(Some((x, y, z)))
        }
        _ => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
    #[serde(default, deserialize_with = "deserialize_grapple")]
    pub grapple: Option<(f64, f64, f64)>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct AiUpdateData {
    pub sid: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub x_d: f64,
    pub y_d: f64,
    pub health: i64,
    pub did_mov: i64,
    pub team: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "ai", allow_extra)]
pub struct AiSyncPacket {
    #[packet(chunks(AiUpdateData, 9, zero_as_empty))]
    pub updates: Vec<AiUpdateData>,
}

#[derive(Debug, Packet)]
#[packet(event = "aai", scalar_as_seq, allow_extra)]
pub struct AddAiPacket {
    pub sid: i64,
    pub team: i64,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub config: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerData {
    pub id: String,
    pub sid: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub alias: String,
    pub class_index: i64,
    pub health: i64,
    pub max_health: i64,
    pub team: Option<i64>,
    pub level: i64,
    pub clan: Value,
    pub skins: Value,
    pub face_index: Value,
    pub shoe_index: Value,
    pub hat_index: Value,
    pub head_index: Value,
    pub body_index: Value,
    pub x_dire: f64,
    pub back_index: Value,
    pub hair_col: Value,
    pub dye_index: Value,
    pub attach_index: Option<i64>,
    pub pc_stat_index: Option<i64>,
    pub sec_index: Option<i64>,
    pub featured: i64,
    pub flag_index: Option<i64>,
    pub premium_t: f64,
    pub display_name: String,
    pub charms: i64,
    pub waist_index: Option<i64>,
    pub accid: i64,
    pub field32: Value,
    pub field33: Value,
    pub field34: Value,
    pub badge_index: i64,
    pub is_bot: i64,
    pub field37: Value,
    pub field38: Value,
    pub field39: Value,
    pub field40: Value,
    pub field41: Value,
    pub field42: Value,
    pub player_card_index: i64,
    pub field44: Value,
    pub busy_value: i64,
    pub field46: Value,
    pub field47: Value,
    pub field48: Value,
    pub field49: Value,
    pub field50: Value,
}

#[derive(Debug, Packet)]
#[packet(event = "0", allow_extra)]
pub struct AddPlayersPacket {
    #[packet(chunks(PlayerData, 51))]
    pub players: Vec<PlayerData>,
    pub flag: u8,
}

#[derive(Debug, Packet)]
#[packet(event = "3", scalar_as_seq, allow_extra)]
pub struct Kill {
    pub victim_sid: i64,
    pub killer_sid: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "start", scalar_as_seq, allow_extra)]
pub struct Start {
    pub timer: String,
    pub state: Option<i64>,
    pub spectating: bool,
    pub challenge_mode: Option<i64>,
    pub movement_lock: i64,
    pub start_flag: i64,
    pub kpd_data: Option<Value>,
    pub class_index: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "f", scalar_as_seq, allow_extra)]
pub struct FailedInput {
    pub issue: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EndStats {
    pub ed: Vec<Value>,
    pub vo: Value,
    pub mts: Vec<String>,
    pub mdls: Vec<Value>,
    #[serde(rename = "modeIndex")]
    pub mode_index: i64,
    pub tms: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
#[serde(default)]
pub struct EndRaidPromo {
    pub items: Vec<Value>,
    #[serde(rename = "showRaidPromo")]
    pub show_raid_promo: bool,
    #[serde(rename = "isDoubleRaidDrops")]
    pub is_double_raid_drops: bool,
}

#[derive(Debug, Packet)]
#[packet(event = "end", scalar_as_seq, allow_extra)]
pub struct EndPacket {
    pub field0: Value,
    pub field1: Option<Value>,
    pub end_stats: EndStats,
    pub end_reason: i64,
    pub kr_reward: Option<i64>,
    pub reward_amount: i64,
    pub bp_data: Option<Value>,
    #[packet(default)]
    pub raid_promo: EndRaidPromo,
}

#[derive(Debug, Packet)]
#[packet(event = "l", scalar_as_seq, allow_extra, default_missing)]
pub struct MeUpdatePacket {
    pub timestamp: i64,
    pub ping: i64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub pos_z: f64,
    pub vel_y: f64,
    pub vel_x: f64,
    pub vel_z: f64,
    pub x_direction: f64,
    pub on_ground: i8,
    pub did_jump: i8,
    pub did_jump_w: i8,
    pub wall_jump_count: i64,
    pub on_wall: i8,
    pub on_ladder: i8,
    pub aim_value: f64,
    pub crouch_value: f64,
    pub weapon_param: f64,
    pub slide_timer: f64,
    pub can_slide: i8,
    pub on_terrain: i8,
    pub x_vel_clamp: f64,
    pub z_vel_clamp: f64,
    pub grapple: Option<f64>,
    pub can_slide_toggle: i64,
    pub protected_data: Option<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "init", scalar_as_seq, allow_extra)]
pub struct Init {
    pub map_index: i64,
    pub mode_index: i64,
    pub event_index: i64,
    pub ranked_team_scores: Value,
    pub team_scores: Value,
    pub host: Value,
    pub config: Value,
    pub custom_data: Value,
    pub mod_url: Value,
    pub custom_map_data: Value,
    pub reserved: Option<Value>,
    pub sync_data: Value,
    pub network_data: Value,
    pub is_live: bool,
    pub is_official: bool,
    pub is_comp: bool,
    pub reserved_flag: bool,
    pub session_id: Value,
    pub round_id: Value,
    pub game_timer: Value,
    pub field21: Value,
}

#[derive(Debug, Packet)]
#[packet(event = "cc", scalar_as_seq, allow_extra)]
pub struct CheatCheck {
    pub seed: String,
    pub request_id: i64,
    pub payload: String,
}

#[derive(Debug, Packet)]
#[packet(event = "h", scalar_as_seq, allow_extra, default_missing)]
pub struct ChangeHealth {
    pub health: f64,
    pub sid: Option<i64>,
    pub damage_src: Option<i64>,
    pub crit: Option<i64>,
    pub damage_type: Option<i64>,
    pub force_max: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "4", scalar_as_seq, allow_extra, default_missing)]
pub struct DoDamage {
    pub target_sid: i64,
    pub damage: f64,
    pub crit: Option<bool>,
    pub headshot: Option<bool>,
    pub bullet_param: Option<Value>,
    pub silent: Option<bool>,
}

#[derive(Debug, Packet)]
#[packet(event = "5", scalar_as_seq, allow_extra, default_missing)]
pub struct GetScore {
    pub score_delta: i64,
    pub suppress_anim: bool,
    pub no_score_update: bool,
    pub class_index: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "6", allow_extra, default_missing)]
pub struct GetKill {
    pub medals: Vec<Value>,
    pub kill_sound: Option<Value>,
    pub total_kills: Option<i64>,
    pub kill_delta: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "10", allow_extra)]
pub struct GetAssist {}

#[derive(Debug, Packet)]
#[packet(event = "9", scalar_as_seq, allow_extra, default_missing)]
pub struct ShowTracer {
    pub sid: i64,
    pub end_x: f64,
    pub end_y: f64,
    pub end_z: f64,
    pub dir_x: Option<f64>,
    pub dir_y: Option<f64>,
    pub bullet_flag: Option<i64>,
    pub start_x: f64,
    pub start_y: f64,
    pub start_h: f64,
    pub start_z: f64,
    pub bullet_type: Option<i64>,
    pub impact_type: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "s", scalar_as_seq, default_missing)]
pub struct PlayerSound {
    pub sound: Option<Value>,
    pub sid: Option<i64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub volume: Option<f64>,
    pub pitch: Option<f64>,
    pub range: Option<f64>,
    pub loop_count: Option<i64>,
    #[packet(extras)]
    pub trailing: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "gmsg", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateGameMessage {
    pub message: Option<Value>,
    pub append: Option<Value>,
    pub countdown_key: Option<Value>,
    pub popup: Option<Value>,
    pub field4: Option<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "wstk", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateWeaponStreak {
    pub weapon_class: i64,
    pub streak: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "pre", scalar_as_seq, allow_extra, default_missing)]
pub struct EndProjectile {
    pub projectile_type: i64,
    pub sid: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub flag: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "chi", scalar_as_seq, allow_extra, default_missing)]
pub struct AddChatI18n {
    pub sender_sid: i64,
    pub name: Option<String>,
    pub i18n_args: Vec<Value>,
    pub chat_type: Option<i64>,
    pub field4: Option<Value>,
    pub field5: Option<Value>,
    pub field6: Option<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "mv", allow_extra, default_missing)]
pub struct UpdateMatchVote {
    pub votes: Option<Value>,
    pub options: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "pr", scalar_as_seq, default_missing)]
pub struct ShowProjectile {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub spread_x: Option<i64>,
    pub spread_y: Option<i64>,
    pub projectile_type: Option<i64>,
    pub params: Option<Value>,
    pub flag: Option<i64>,
    #[packet(extras)]
    pub trailing: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "gsc", scalar_as_seq, allow_extra, default_missing)]
pub struct GameStateChanged {
    pub state: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "ana", scalar_as_seq, allow_extra, default_missing)]
pub struct ServAnim {
    pub sid: i64,
    pub anim_id: i64,
    pub flag: Option<i64>,
    // JS: `if (IÍìíïîí)` — true = real player, false = AI
    pub is_player: Option<bool>,
}

#[derive(Debug, Packet)]
#[packet(event = "kst", scalar_as_seq, allow_extra, default_missing)]
pub struct KillStreakM {
    pub sid: i64,
    pub streak: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "ex", scalar_as_seq, allow_extra, default_missing)]
pub struct Explosion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub radius: f64,
    pub sid: Option<i64>,
    pub field5: Option<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "rai", scalar_as_seq, allow_extra)]
pub struct RemoveAi {
    pub sid: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "unb", scalar_as_seq, allow_extra, default_missing)]
pub struct UnboxMessage {
    pub item_name: String,
    pub item_id: i64,
    pub multi: bool,
}

#[derive(Debug, Packet)]
#[packet(event = "abd", allow_extra, default_missing)]
pub struct AllBundleData {
    pub bundles: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "ger", allow_extra, default_missing)]
pub struct GuestEarnedRewards {}

#[derive(Debug, Packet)]
#[packet(event = "sb", scalar_as_seq, default_missing)]
pub struct ShowSpeechBubble {
    pub message: Option<Value>,
    #[packet(extras)]
    pub trailing: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "inat", scalar_as_seq, default_missing)]
pub struct PlayerInteractions {
    pub sid: i64,
    #[packet(extras)]
    pub fields: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "warsClan", scalar_as_seq, allow_extra, default_missing)]
pub struct WarsClan {
    pub clan_id: i64,
    pub clan_name: String,
    pub clan_logo: Option<i64>,
    pub clan_banner_color: Option<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "00", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateNames {
    pub updates: Vec<Value>,
    pub keep_connection: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "ch", scalar_as_seq, default_missing)]
pub struct AddChat {
    pub team: i64,
    pub name: Option<String>,
    pub message: Option<String>,
    pub chat_type: Option<i64>,
    pub color: Option<Value>,
    pub badge_or_filter: Option<Value>,
    pub account_id: Option<i64>,
    #[packet(extras)]
    pub trailing: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "crsp", scalar_as_seq, default_missing)]
pub struct CreateSpawnable {
    pub spawn_type: i64,
    pub sid: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[packet(extras)]
    pub trailing: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "do", scalar_as_seq, allow_extra)]
pub struct DestroyGameObject {
    pub id: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "dsp", scalar_as_seq, allow_extra)]
pub struct DisposeSpawnable {
    pub sid: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "gte", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateGate {
    pub id: i64,
    pub open_state: bool,
}

#[derive(Debug, Packet)]
#[packet(event = "lv", scalar_as_seq, allow_extra)]
pub struct UpdateLives {
    pub lives: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "upk", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateZombiePerks {
    pub perk_count: i64,
    pub perks: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "uchp", allow_extra, default_missing)]
pub struct UpdateChallengesProgress {
    pub progress: Vec<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "gt", scalar_as_seq, allow_extra, default_missing)]
pub struct LogTime {
    pub time: i64,
    pub flag: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "chp", allow_extra)]
pub struct CheckPointSet {}

#[derive(Debug, Packet)]
#[packet(event = "chrg", scalar_as_seq, allow_extra, default_missing)]
pub struct SetPlayerCharged {
    pub sid: i64,
    pub time_ms: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "am", allow_extra, default_missing)]
pub struct AddMedal {
    pub medal: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "sp", scalar_as_seq, default_missing)]
pub struct AddSpray {
    pub spray_id: String,
    pub sid: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rot: f64,
    #[packet(extras)]
    pub trailing: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BpReward {
    pub bpp: i64,
    pub jnk: f64,
    pub kr: Option<i64>,
    pub sk: Option<Vec<i64>>,
}

#[derive(Debug, Packet)]
#[packet(event = "chgC", scalar_as_seq, allow_extra, default_missing)]
pub struct ChallengeCompleted {
    pub challenge_id: i64,
    pub complete: bool,
    pub rewards: Option<BpReward>,
}

#[derive(Debug, Packet)]
#[packet(event = "bpdl", map)]
pub struct DataLoaded {
    #[packet(default)]
    pub id: i64,
    #[packet(default)]
    #[packet(rename = "bpp_perlevel")]
    pub bpp_per_level: i64,
    #[packet(default)]
    pub descr: String,
    #[packet(default)]
    pub enddate: String,
    #[packet(default)]
    pub itemcount: i64,
    #[packet(default)]
    pub itemdata: String,
}

#[derive(Debug, Packet)]
#[packet(event = "bppd", map)]
pub struct SyncBpProg {
    #[packet(default)]
    pub bpp: i64,
    #[packet(default)]
    pub tier: i64,
    #[packet(default)]
    pub items_claimed: String,
}

#[derive(Debug, Packet)]
#[packet(event = "gbdr", allow_extra, default_missing)]
pub struct AvailableBundleData {
    pub bundle_ids: Vec<i64>,
    pub field1: Vec<Value>,
    pub field2: Option<Value>,
    pub bundles: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "gfrnd", scalar_as_seq, allow_extra, default_missing)]
pub struct FriendsListResponse {
    pub friends: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "gmc", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateMail {
    pub count: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "chg", allow_extra, default_missing)]
pub struct UpdateChallenges {
    pub challenges: Vec<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "cnfm", scalar_as_seq, allow_extra)]
pub struct ConfirmInteraction {
    pub interaction_id: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "cust", scalar_as_seq, allow_extra, default_missing)]
pub struct CustomResponse {
    pub message: String,
    pub code: Option<String>,
    pub flag: Option<bool>,
}

#[derive(Debug, Packet)]
#[packet(event = "cv", scalar_as_seq, allow_extra, default_missing)]
pub struct CustomVal {
    pub key: String,
    pub value: Option<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "ua", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateAccount {
    pub profile: Option<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "uf", scalar_as_seq, allow_extra)]
pub struct UpdateFunds {
    pub funds: i64,
}

#[derive(Debug, Packet)]
#[packet(event = "chlR", allow_extra, default_missing)]
pub struct UpdateChallengeRewards {
    pub rewards: Vec<Value>,
    pub funds: Option<i64>,
    pub junk: Option<f64>,
    pub field3: Option<f64>,
    pub field4: Option<i64>,
}

#[derive(Debug, Packet)]
#[packet(event = "ulb", scalar_as_seq, allow_extra, default_missing)]
pub struct UpdateLeaderBoard {
    pub state: i64,
    pub entries: Vec<Value>,
}

#[derive(Debug, EventEnum)]
pub enum Event {
    CheatCheck(CheatCheck),
    Leaderboard(LeaderboardPacket),
    Time(Time),
    End(EndPacket),
    Start(Start),
    Cntry(Country),
    IoInit(IoInit),
    Init(Init),
    RemovePlayer(RemovePlayer),
    Error(Error),
    Pi(Ping),
    Pir(PingReturn),
    Ready(Ready),
    Remail(ReMail),
    Load(Load),
    UID(UID),
    InstID(InstID),
    Captcha(Captcha),
    PlayerUpdate(PlayerUpdate),
    AddPlayers(AddPlayersPacket),
    Kill(Kill),
    UpdateMe(MeUpdatePacket),
    FailedInput(FailedInput),
    AiSyncPacket(AiSyncPacket),
    AddAiPacket(AddAiPacket),
    Test(Test),
    AccountAuth(AccountAuth),
    ChangeHealth(ChangeHealth),
    DoDamage(DoDamage),
    GetScore(GetScore),
    GetKill(GetKill),
    GetAssist(GetAssist),
    ShowTracer(ShowTracer),
    PlayerSound(PlayerSound),
    UpdateGameMessage(UpdateGameMessage),
    UpdateWeaponStreak(UpdateWeaponStreak),
    EndProjectile(EndProjectile),
    AddChatI18n(AddChatI18n),
    UpdateMatchVote(UpdateMatchVote),
    ShowProjectile(ShowProjectile),
    GameStateChanged(GameStateChanged),
    ServAnim(ServAnim),
    KillStreakM(KillStreakM),
    Explosion(Explosion),
    RemoveAi(RemoveAi),
    UnboxMessage(UnboxMessage),
    AllBundleData(AllBundleData),
    GuestEarnedRewards(GuestEarnedRewards),
    ShowSpeechBubble(ShowSpeechBubble),
    PlayerInteractions(PlayerInteractions),
    WarsClan(WarsClan),
    UpdateNames(UpdateNames),
    AddChat(AddChat),
    CreateSpawnable(CreateSpawnable),
    DestroyGameObject(DestroyGameObject),
    DisposeSpawnable(DisposeSpawnable),
    UpdateGate(UpdateGate),
    UpdateLives(UpdateLives),
    UpdateZombiePerks(UpdateZombiePerks),
    UpdateChallengesProgress(UpdateChallengesProgress),
    LogTime(LogTime),
    CheckPointSet(CheckPointSet),
    SetPlayerCharged(SetPlayerCharged),
    AddMedal(AddMedal),
    AddSpray(AddSpray),
    ChallengeCompleted(ChallengeCompleted),
    DataLoaded(DataLoaded),
    SyncBpProg(SyncBpProg),
    AvailableBundleData(AvailableBundleData),
    FriendsListResponse(FriendsListResponse),
    UpdateMail(UpdateMail),
    UpdateChallenges(UpdateChallenges),
    ConfirmInteraction(ConfirmInteraction),
    CustomResponse(CustomResponse),
    CustomVal(CustomVal),
    UpdateAccount(UpdateAccount),
    UpdateFunds(UpdateFunds),
    UpdateChallengeRewards(UpdateChallengeRewards),
    UpdateLeaderBoard(UpdateLeaderBoard),
    #[event(unknown)]
    Unknown(String, Vec<serde_value::Value>),
}

#[derive(Debug, Packet)]
#[packet(event = "testPacket", scalar_as_seq, allow_extra)]
pub struct Test {
    #[packet(coerce(num_to_bool, bool_to_num))]
    pub test: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedItem {
    pub ind: i64,
    pub cnt: i64,
    pub st: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccountStats {
    pub anp: i64,
    pub r3: i64,
    pub c: i64,
    pub r2: i64,
    pub crc: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ChallengeEntry {
    pub ic: i64,
    pub name: String,
    pub challenge_type: String,
    pub data: ChallengeData,
    pub reward: ChallengeReward,
    pub tr: i64,
    pub ci: i64,
    pub pt: i64,
    pub pv: i64,
    pub sv: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ChallengeData {
    pub ky: String,
    pub stk: Option<i64>,
    pub mp: Option<i64>,
    pub val: i64,
    pub boss: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChallengeReward {
    pub kr: Option<i64>,
    pub sk: Option<Vec<i64>>,
    pub jnk: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileData {
    pub account_id: i64,
    pub kills: i64,
    pub wins: i64,
    pub games: i64,
    pub deaths: i64,
    pub funds: i64,
    pub score: i64,
    pub clan: Option<String>,
    pub time_played: i64,
    pub featured: i64,
    pub skins: Vec<OwnedItem>,
    pub last_reward: i64,
    pub is_dev: i64,
    pub following: i64,
    pub followers: i64,
    pub stats: AccountStats,
    pub hack: i64,
    pub region_ind: i64,
    pub role: i64,
    pub premium_t: i64,
    pub two_factor_auth: bool,
    pub alias: Option<String>,
    pub create_date: String,
    pub twitch: Option<String>,
    pub is_creation_mod: i64,
    pub clan_deaths: i64,
    pub clan_kills: i64,
    pub job_rating: i64,
    pub job_rating_pos: i64,
    pub req_del: bool,
    pub clan_role: i64,
    pub clan_time: i64,
    pub external_token: Option<String>,
    pub trades: i64,
    pub email: Option<String>,
    pub is_admin: i64,
    pub inv_val: i64,
    #[serde(deserialize_with = "deserialize_f64_lenient")]
    pub junk: f64,
    pub active_challenges: Vec<ChallengeEntry>,
    pub badges: Vec<i64>,
    pub metamask: Option<String>,
    pub bpp: i64,
    pub is_skin_maker: i64,
    pub email_verified: bool,
    pub twitch_id: Option<String>,
    pub twitch_at: Option<String>,
    pub twitch_rt: Option<String>,
    pub ranked_points: i64,
    pub spending_tier: i64,
    pub subscriptions: Vec<Value>,
    pub clan_logo: i64,
    pub clan_id: i64,
    pub raid_stats: Vec<Value>,
}

#[derive(Debug, Packet)]
#[packet(event = "a", scalar_as_seq, allow_extra, default_missing)]
pub struct AccountAuth {
    pub error: i64,
    pub account_id: i64,
    pub username: String,
    pub profile: ProfileData,
    pub token: Option<String>,
    pub extra: Option<Value>,
}
