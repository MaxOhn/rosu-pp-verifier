use std::{fmt, mem};

use rkyv::with::Skip;

use crate::MAX_ALIGN;

use super::{
    difficulty::{
        CatchDifficultyAttributes, ManiaDifficultyAttributes, OsuDifficultyAttributes,
        TaikoDifficultyAttributes,
    },
    mode::{Catch, IGameMode, Mania, Osu, Taiko},
    mods::GameMods,
    performance::{
        CatchPerformanceAttributes, ManiaPerformanceAttributes, OsuPerformanceAttributes,
        TaikoPerformanceAttributes,
    },
    simulate_score::SimulateScore,
    statistics::Statistics,
};

#[derive(serde::Deserialize)]
pub struct SimulateObject<M: IGameMode> {
    pub score: SimulateScore,
    performance_attributes: M::Performance,
    difficulty_attributes: M::Difficulty,
}

impl<M: IGameMode> fmt::Debug for SimulateObject<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SimulateObject")
            .field("score", &self.score)
            .field("performance_attributes", &self.performance_attributes)
            .field("difficulty_attributes", &self.difficulty_attributes)
            .finish()
    }
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(
    archived = ArchivedSimulateObject,
    resolver = SimulateObjectResolver,
    attr(derive(Debug)),
)]
pub struct ArchivableSimulateObject {
    pub beatmap_id: i32,
    pub mods: GameMods,
    #[rkyv(with = Skip)]
    pub statistics: Statistics,
    pub attrs: SimulateAttributes,
}

impl ArchivedSimulateObject {
    pub fn mode(&self) -> &'static str {
        match self.attrs {
            ArchivedSimulateAttributes::Osu { .. } => "Osu",
            ArchivedSimulateAttributes::Taiko { .. } => "Taiko",
            ArchivedSimulateAttributes::Catch { .. } => "Catch",
            ArchivedSimulateAttributes::Mania { .. } => "Mania",
        }
    }
}

const_assert!(mem::align_of::<ArchivedSimulateObject>() <= MAX_ALIGN);

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(attr(derive(Debug)))]
pub enum SimulateAttributes {
    Osu {
        perf: OsuPerformanceAttributes,
        diff: OsuDifficultyAttributes,
    },
    Taiko {
        perf: TaikoPerformanceAttributes,
        diff: TaikoDifficultyAttributes,
    },
    Catch {
        perf: CatchPerformanceAttributes,
        diff: CatchDifficultyAttributes,
    },
    Mania {
        perf: ManiaPerformanceAttributes,
        diff: ManiaDifficultyAttributes,
    },
}

macro_rules! impl_from {
    ( $mode:ident ) => {
        impl From<SimulateObject<$mode>> for ArchivableSimulateObject {
            fn from(attrs: SimulateObject<$mode>) -> Self {
                Self {
                    beatmap_id: attrs.score.beatmap_id,
                    mods: attrs.score.mods,
                    statistics: attrs.score.statistics,
                    attrs: SimulateAttributes::$mode {
                        perf: attrs.performance_attributes.into(),
                        diff: attrs.difficulty_attributes.into(),
                    },
                }
            }
        }
    };
}

impl_from!(Osu);
impl_from!(Taiko);
impl_from!(Catch);
impl_from!(Mania);
