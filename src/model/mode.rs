use std::fmt::Debug;

use super::{
    difficulty::{
        CatchRawDifficultyAttributes, ManiaRawDifficultyAttributes, OsuRawDifficultyAttributes,
        TaikoRawDifficultyAttributes,
    },
    performance::{
        CatchRawPerformanceAttributes, ManiaRawPerformanceAttributes, OsuRawPerformanceAttributes,
        TaikoRawPerformanceAttributes,
    },
};

pub struct Osu;
pub struct Taiko;
pub struct Catch;
pub struct Mania;

pub trait IGameMode {
    const ARG: &'static str;
    const MODS: &'static [&'static str];

    type Performance: Debug + for<'de> serde::Deserialize<'de>;
    type Difficulty: Debug + for<'de> serde::Deserialize<'de>;
}

macro_rules! impl_mode {
    ( $mode:ident: $arg:ident, $performance:ident, $difficulty:ident, [ $( $gamemods:ident ),* ] ) => {
        impl IGameMode for $mode {
            const ARG: &'static str = stringify!($arg);
            const MODS: &'static [&'static str] = &[$( stringify!($gamemods) ),* ];

            type Performance = $performance;
            type Difficulty = $difficulty;
        }
    };
}

impl_mode!(Osu: osu, OsuRawPerformanceAttributes, OsuRawDifficultyAttributes, [NM, HD, EZHD, HR, DT, FL, HDFL, HRDT, HDDTFL]);
impl_mode!(Taiko: taiko, TaikoRawPerformanceAttributes, TaikoRawDifficultyAttributes, [NM, HD, HR, DT]);
impl_mode!(Catch: catch, CatchRawPerformanceAttributes, CatchRawDifficultyAttributes, [NM, HD, HR, HDHR, EZ, DT]);
impl_mode!(Mania: mania, ManiaRawPerformanceAttributes, ManiaRawDifficultyAttributes, [NM, EZ, DT]);
