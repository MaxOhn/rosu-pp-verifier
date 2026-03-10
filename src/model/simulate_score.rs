use super::{mods::GameMods, statistics::Statistics};

#[derive(Debug, serde::Deserialize)]
pub struct SimulateScore {
    pub beatmap_id: i32,
    pub mods: GameMods,
    pub statistics: Statistics,
}
