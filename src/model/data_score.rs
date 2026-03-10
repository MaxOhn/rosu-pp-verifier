use std::{fmt, marker::PhantomData, mem};

use rkyv::with::Skip;
use serde::{
    de::{DeserializeOwned, Error, IgnoredAny, Visitor},
    Deserializer,
};

use crate::MAX_ALIGN;

use super::{mods::GameMods, statistics::Statistics};

pub struct JsonData<T>(PhantomData<T>);

impl<T: DeserializeOwned> JsonData<T> {
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<T, D::Error> {
        d.deserialize_str(JsonData(PhantomData))
    }
}

impl<T: DeserializeOwned> Visitor<'_> for JsonData<T> {
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("JSON data")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        serde_json::from_str(&v.replace("\\\"", "\"")).map_err(Error::custom)
    }
}

const_assert!(mem::align_of::<ArchivedDataScore>() <= MAX_ALIGN);

#[derive(serde::Deserialize, rkyv::Archive, rkyv::Serialize)]
pub struct DataScore {
    pub id: u64,
    #[rkyv(with = Skip)]
    pub user_id: IgnoredAny,
    #[serde(rename = "ruleset_id")]
    pub mode: u8,
    #[serde(rename = "beatmap_id")]
    pub map_id: u32,
    #[rkyv(with = Skip)]
    pub has_replay: IgnoredAny,
    #[rkyv(with = Skip)]
    pub preserve: IgnoredAny,
    #[rkyv(with = Skip)]
    pub ranked: IgnoredAny,
    #[rkyv(with = Skip)]
    pub rank: IgnoredAny,
    #[rkyv(with = Skip)]
    pub passed: IgnoredAny,
    #[serde(rename = "accuracy")]
    pub acc: f32,
    #[serde(rename = "max_combo")]
    pub combo: u32,
    #[rkyv(with = Skip)]
    pub total_score: IgnoredAny,
    #[serde(deserialize_with = "JsonData::<Data>::deserialize")]
    pub data: Data,
    #[rkyv(with = Skip)]
    pub pp: IgnoredAny,
    #[rkyv(with = Skip)]
    pub legacy_score_id: IgnoredAny,
    #[rkyv(with = Skip)]
    pub legacy_total_score: IgnoredAny,
    #[rkyv(with = Skip)]
    pub started_at: IgnoredAny,
    #[rkyv(with = Skip)]
    pub ended_at: IgnoredAny,
    #[rkyv(with = Skip)]
    pub unix_updated_at: IgnoredAny,
    #[serde(rename = "build_id", deserialize_with = "deserialize_build_id")]
    pub lazer: bool,
    #[serde(default)]
    pub checked: bool,
}

fn deserialize_build_id<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    Ok(<&str as serde::Deserialize<'de>>::deserialize(d)? != "NULL")
}

impl fmt::Debug for ArchivedDataScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Score")
            .field("id", &self.id)
            .field("mode", &self.mode)
            .field("map_id", &self.map_id)
            .field("acc", &self.acc)
            .field("combo", &self.combo)
            .field("mods", &self.data.mods)
            .field("stats", &self.data.stats)
            .field("max_stats", &self.data.max_stats)
            .field("lazer", &self.lazer)
            .finish()
    }
}

#[derive(serde::Deserialize, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug))]
pub struct Data {
    pub mods: GameMods,
    #[serde(rename = "statistics")]
    pub stats: Statistics,
    #[serde(rename = "maximum_statistics")]
    pub max_stats: Statistics,
}
