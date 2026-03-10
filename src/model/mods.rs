use std::fmt;

use rkyv::{rancor::{Fallible, Source}, ser::Writer, string::{ArchivedString, StringResolver}, with::{ArchiveWith,  SerializeWith}, Place};
use serde_json::Number;
use serde::de::{
    value::MapAccessDeserializer, Deserialize, Deserializer, Error, MapAccess, Visitor,
};

#[derive(serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GameMods(Box<[GameMod]>);

impl fmt::Debug for GameMods {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("NM");
        }

        for gamemod in self.0.iter() {
            fmt::Debug::fmt(gamemod, f)?;
        }

        Ok(())
    }
}

impl ArchivedGameMods {
    pub fn acronyms(&self) -> impl Iterator<Item = &'_ str> {
        self.0.iter().map(|m| m.acronym.as_ref())
    }

    pub fn bits(&self) -> u32 {
        self.0.iter().fold(0, |acc, val| acc | val.bits())
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ ArchivedGameMod> {
        self.0.iter()
    }

    pub fn varying_clock_rate(&self) -> bool {
        self.0.iter().any(|m| matches!(m.acronym.as_ref(), "WU" | "WD" | "AS"))
    }
    
    

    pub fn contains_acronym(&self, acronym: &str) -> bool {
        self.acronyms().any(|a| a == acronym)
    }

    pub fn contains_classic(&self) -> bool {
        self.contains_acronym("CL")
    }
    
    pub fn contains_random(&self) -> bool {
        self.contains_acronym("RD")
    }
    
    pub fn convert(&self, mode: rosu_mods::GameMode) -> Result<rosu_mods::GameMods, String> {
        self.iter()
            .map(|m| {
                let mut gamemod = rosu_mods::GameMod::new(&m.acronym, mode);

                for setting in m.settings.iter() {
                    let key = setting.key.as_ref();

                    macro_rules! mutate_gamemod {
                        ( $(
                            [ $( $variant:ident $(,)?  )* ] => [ $( $key:ident $(,)? )* ];
                        )* ) => {
                            mutate_gamemod!(1 $( $( $variant ),* => { $( $key ),* }; )* );
                        };
                        (1 $( $( $variant:ident ),* => $keys:tt; )* ) => {
                            mutate_gamemod!(2 $( $( $variant => $keys, )* )* );
                        };
                        (2 $( $variant:ident => { $( $key:ident ),* }, )* ) => {
                            match &mut gamemod {
                                $(
                                    rosu_mods::GameMod::$variant(m) => match key {
                                        $( stringify!($key) => {
                                            m.$key = Some(
                                                setting.value.as_str().parse().unwrap()
                                            );
                                        }, )*
                                        _ => return Err(format!(
                                            concat!("unknown setting `{}` for ", stringify!($variant)),
                                            key,
                                        )),
                                    },
                                )*
                                rosu_mods::GameMod::UnknownOsu(_)
                                | rosu_mods::GameMod::UnknownTaiko(_)
                                | rosu_mods::GameMod::UnknownCatch(_)
                                | rosu_mods::GameMod::UnknownMania(_) => {
                                    return Err(format!("unknown acronym {}", m.acronym));
                                }
                                _ => return Err(format!(
                                    "setting `{:?}` is not handled for {}",
                                    setting,
                                    m.acronym,
                                )),
                            }
                        };
                    }
                    
                    mutate_gamemod! {
                        [AccuracyChallengeOsu, AccuracyChallengeTaiko, AccuracyChallengeCatch, AccuracyChallengeMania] => [accuracy_judge_mode, minimum_accuracy, restart];
                        [AdaptiveSpeedOsu, AdaptiveSpeedTaiko, AdaptiveSpeedMania] => [adjust_pitch, initial_rate];
                        [ApproachDifferentOsu] => [scale, style];
                        [BarrelRollOsu] => [direction, spin_speed];
                        [ClassicOsu] => [always_play_tail_sample, classic_health, classic_note_lock, fade_hit_circle_early, no_slider_head_accuracy];
                        [CoverMania] => [coverage, direction];
                        [DaycoreOsu, DaycoreTaiko, DaycoreCatch, DaycoreMania] => [speed_change];
                        [DeflateOsu] => [start_scale];
                        [DepthOsu] => [max_depth, show_approach_circles];
                        [DifficultyAdjustOsu] => [approach_rate, circle_size, drain_rate, extended_limits, overall_difficulty];
                        [DifficultyAdjustCatch] => [approach_rate, circle_size, drain_rate, extended_limits, hard_rock_offsets, overall_difficulty];
                        [DifficultyAdjustTaiko] => [drain_rate, extended_limits, overall_difficulty, scroll_speed];
                        [DifficultyAdjustMania] => [drain_rate, extended_limits, overall_difficulty];
                        [DoubleTimeOsu, DoubleTimeTaiko, DoubleTimeCatch, DoubleTimeMania] => [adjust_pitch, speed_change];
                        [EasyOsu, EasyCatch, EasyMania] => [retries];
                        [FlashlightOsu] => [combo_based_size, follow_delay, size_multiplier];
                        [FlashlightTaiko, FlashlightCatch, FlashlightMania] => [combo_based_size, size_multiplier];
                        [GrowOsu] => [start_scale];
                        [HalfTimeOsu, HalfTimeTaiko, HalfTimeCatch, HalfTimeMania] => [adjust_pitch, speed_change];
                        [HiddenOsu] => [only_fade_approach_circles];
                        // [HiddenMania] => [coverage]; (deprecated)
                        [MagnetisedOsu] => [attraction_strength];
                        [MirrorOsu] => [reflection];
                        [MutedOsu, MutedTaiko, MutedCatch, MutedMania] => [affects_hit_sounds, enable_metronome, inverse_muting, mute_combo_count];
                        [NightcoreOsu, NightcoreTaiko, NightcoreCatch, NightcoreMania] => [speed_change];
                        [NoScopeOsu,  NoScopeCatch] => [hidden_combo_count];
                        [PerfectOsu, PerfectTaiko, PerfectCatch, PerfectMania] => [restart];
                        [RandomOsu] => [angle_sharpness, seed];
                        [RandomTaiko, RandomMania] => [seed];
                        [RepelOsu] => [repulsion_strength];
                        [SuddenDeathOsu, SuddenDeathTaiko, SuddenDeathCatch, SuddenDeathMania] => [restart];
                        [TargetPracticeOsu] => [seed];
                        [WiggleOsu] => [strength];
                        [WindUpOsu, WindUpTaiko, WindUpCatch, WindUpMania] => [adjust_pitch, final_rate, initial_rate];
                        [WindDownOsu, WindDownTaiko, WindDownCatch, WindDownMania] => [adjust_pitch, final_rate, initial_rate];
                    };
                }

                Ok(gamemod)
            })
            .collect()
    }
}

impl fmt::Debug for ArchivedGameMods {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("NM");
        }

        for gamemod in self.0.iter() {
            fmt::Debug::fmt(gamemod, f)?;
        }

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GameMod {
    pub acronym: Box<str>,
    #[serde(default, deserialize_with = "GameModSetting::deserialize_all")]
    pub settings: Box<[GameModSetting]>,
}


impl ArchivedGameMod {
    pub fn bits(&self) -> u32 {
        match self.acronym.as_ref() {
            "NF" => 1 << 0,
            "EZ" => 1 << 1,
            "TD" => 1 << 2,
            "HD" => 1 << 3,
            "HR" => 1 << 4,
            "DT" => 1 << 6,
            "HT" => 1 << 8,
            "FL" => 1 << 10,
            "SO" => 1 << 12,
            other => {
                println!("Unknown mod {other:?}");

                0
            }
        }
    }
}

impl fmt::Debug for ArchivedGameMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.acronym)?;

        if !self.settings.is_empty() {
            f.write_str("[")?;
            let mut comma = false;

            for setting in self.settings.iter() {
                if comma {
                    f.write_str(", ")?;
                }

                comma = true;
                write!(f, "{}={}", setting.key, setting.value.as_str())?;
            }

            f.write_str("]")?;
        }

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug))]
pub struct GameModSetting {
    pub key: Box<str>,
    pub value: GameModSettingValue,
}


impl GameModSetting {
    fn deserialize_all<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Box<[GameModSetting]>, D::Error> {
        struct SettingsVisitor;

        impl<'de> Visitor<'de> for SettingsVisitor {
            type Value = Box<[GameModSetting]>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("settings map")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut settings = Vec::with_capacity(map.size_hint().unwrap_or(1));

                while let Some((key, value)) = map.next_entry()? {
                    settings.push(GameModSetting { key, value });
                }

                Ok(settings.into_boxed_slice())
            }
        }

        d.deserialize_map(SettingsVisitor)
    }
}

#[derive(Debug, rkyv::Archive, rkyv::Serialize)]
#[rkyv(derive(Debug))]
pub enum GameModSettingValue {
    Number(#[rkyv(with = AsStr)] Number),
    Bool(bool),
    Str(Box<str>),
}

struct AsStr;

impl ArchiveWith<Number> for AsStr {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    fn resolve_with(
        n: &Number,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        ArchivedString::resolve_from_str(n.as_str(), resolver, out);
    }
}

impl<S: Fallible<Error: Source> + Writer> SerializeWith<Number, S> for AsStr {
    fn serialize_with(
        n: &Number,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(n.as_str(), serializer)
    }
}

impl ArchivedGameModSettingValue {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Number(value) => value.as_str(),
            Self::Bool(true) => "true",
            Self::Bool(false) => "false",
            Self::Str(value) => value,
        }
    }
}


    impl<'de> Deserialize<'de> for GameModSettingValue {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct ValueVisitor;

            impl<'de> Visitor<'de> for ValueVisitor {
                type Value = GameModSettingValue;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("a number, boolean, or string")
                }

                fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
                    Ok(GameModSettingValue::Bool(v))
                }

                fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                    Ok(GameModSettingValue::Number(v.into()))
                }

                fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
                    Ok(GameModSettingValue::Number(v.into()))
                }

                fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
                    Ok(GameModSettingValue::Number(
                        serde_json::Number::from_f64(v).unwrap(),
                    ))
                }

                fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                    Ok(GameModSettingValue::Str(Box::from(v)))
                }

                fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                    Number::deserialize(MapAccessDeserializer::new(map))
                        .map(GameModSettingValue::Number)
                        .map_err(Error::custom)
                }
            }

            d.deserialize_any(ValueVisitor)
        }
    }