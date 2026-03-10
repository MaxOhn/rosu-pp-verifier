use std::fmt;

use crate::model::{
    difficulty::{
        ArchivedCatchDifficultyAttributes, ArchivedManiaDifficultyAttributes,
        ArchivedOsuDifficultyAttributes, ArchivedTaikoDifficultyAttributes,
        CatchDifficultyAttributes, ManiaDifficultyAttributes, OsuDifficultyAttributes,
        TaikoDifficultyAttributes,
    },
    performance::{
        ArchivedCatchPerformanceAttributes, ArchivedManiaPerformanceAttributes,
        ArchivedOsuPerformanceAttributes, ArchivedTaikoPerformanceAttributes,
        CatchPerformanceAttributes, ManiaPerformanceAttributes, OsuPerformanceAttributes,
        TaikoPerformanceAttributes,
    },
};

pub struct NotEq {
    field: &'static str,
    values: Values,
}

impl fmt::Display for NotEq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "field={} {}", self.field, self.values)
    }
}

enum Values {
    F64(ValuesInner<f64>),
    U32(ValuesInner<u32>),
    OptionF64(ValuesInner<Option<f64>>),
}

impl Values {
    fn new_f64(expected: f64, actual: f64) -> Self {
        Values::F64(ValuesInner { expected, actual })
    }

    fn new_option_f64(expected: Option<f64>, actual: Option<f64>) -> Self {
        Values::OptionF64(ValuesInner { expected, actual })
    }

    fn new_u32(expected: u32, actual: u32) -> Self {
        Values::U32(ValuesInner { expected, actual })
    }
}

struct ValuesInner<T> {
    expected: T,
    actual: T,
}

impl fmt::Display for Values {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Values::F64(values) => fmt::Display::fmt(values, f),
            Values::U32(values) => fmt::Display::fmt(values, f),
            Values::OptionF64(values) => fmt::Debug::fmt(values, f),
        }
    }
}

impl<T: fmt::Display> fmt::Display for ValuesInner<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "actual={} expected={}", self.actual, self.expected)
    }
}

impl<T: fmt::Debug> fmt::Debug for ValuesInner<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "actual={:?} expected={:?}", self.actual, self.expected)
    }
}

pub trait AssertEq<Perf, Diff> {
    fn assert_eq(&self, perf: &Perf, diff: &Diff) -> Result<(), NotEq>;
}

impl AssertEq<OsuPerformanceAttributes, OsuDifficultyAttributes>
    for rosu_pp::osu::OsuPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &OsuPerformanceAttributes,
        diff: &OsuDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let OsuPerformanceAttributes {
            aim,
            speed,
            accuracy,
            flashlight,
            effective_miss_count,
            speed_deviation,
            combo_based_estimated_miss_count: _,
            score_based_estimated_miss_count: _,
            aim_estimated_slider_breaks: _,
            speed_estimated_slider_breaks: _,
            pp,
        } = perf;

        let OsuDifficultyAttributes {
            star_rating,
            max_combo,
            aim_difficulty,
            aim_difficult_slider_count,
            speed_difficulty,
            speed_note_count,
            slider_factor,
            aim_difficult_strain_count,
            speed_difficult_strain_count,
            flashlight_difficulty,
            aim_top_weighted_slider_factor: _,
            speed_top_weighted_slider_factor: _,
            nested_score_per_object: _,
            legacy_score_base_multiplier: _,
            maximum_legacy_combo_score,
        } = diff;

        assert_eq_f64(&self.difficulty.stars, *star_rating, "star_rating")?;
        assert_eq_u32(&self.difficulty.max_combo, *max_combo, "max_combo")?;
        assert_eq_f64(&self.difficulty.aim, *aim_difficulty, "aim_difficulty")?;
        assert_eq_f64(
            &self.difficulty.speed,
            *speed_difficulty,
            "speed_difficulty",
        )?;
        assert_eq_f64(
            &self.difficulty.speed_note_count,
            *speed_note_count,
            "speed_note_count",
        )?;
        assert_eq_f64(
            &self.difficulty.aim_difficult_slider_count,
            *aim_difficult_slider_count,
            "aim_difficult_slider_count",
        )?;
        assert_eq_f64(
            &self.difficulty.aim_difficult_strain_count,
            *aim_difficult_strain_count,
            "aim_difficult_strain_count",
        )?;
        assert_eq_f64(
            &self.difficulty.speed_difficult_strain_count,
            *speed_difficult_strain_count,
            "speed_difficult_strain_count",
        )?;
        if let Some(flashlight_difficulty) = flashlight_difficulty.as_ref() {
            assert_eq_f64(
                &self.difficulty.flashlight,
                *flashlight_difficulty,
                "flashlight_difficulty",
            )?;
        }
        assert_eq_f64(
            &self.difficulty.slider_factor,
            *slider_factor,
            "slider_factor",
        )?;
        assert_eq_f64(
            &self.difficulty.maximum_legacy_combo_score,
            *maximum_legacy_combo_score,
            "maximum_legacy_combo_score",
        )?;

        assert_eq_f64(&self.pp, *pp, "pp")?;
        assert_eq_f64(&self.pp_acc, *accuracy, "pp_acc")?;
        assert_eq_f64(&self.pp_aim, *aim, "pp_aim")?;
        assert_eq_f64(&self.pp_flashlight, *flashlight, "pp_flashlight")?;
        assert_eq_f64(&self.pp_speed, *speed, "pp_speed")?;
        assert_eq_f64(
            &self.effective_miss_count,
            *effective_miss_count,
            "effective_miss_count",
        )?;
        assert_eq_option_f64(
            self.speed_deviation.as_ref(),
            *speed_deviation,
            "speed_deviation",
        )?;

        Ok(())
    }
}

impl AssertEq<ArchivedOsuPerformanceAttributes, ArchivedOsuDifficultyAttributes>
    for rosu_pp::osu::OsuPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &ArchivedOsuPerformanceAttributes,
        diff: &ArchivedOsuDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let ArchivedOsuPerformanceAttributes {
            aim,
            speed,
            accuracy,
            flashlight,
            effective_miss_count,
            speed_deviation,
            combo_based_estimated_miss_count: _,
            score_based_estimated_miss_count: _,
            aim_estimated_slider_breaks: _,
            speed_estimated_slider_breaks: _,
            pp,
        } = perf;

        let ArchivedOsuDifficultyAttributes {
            star_rating,
            max_combo,
            aim_difficulty,
            aim_difficult_slider_count,
            speed_difficulty,
            speed_note_count,
            slider_factor,
            aim_difficult_strain_count,
            speed_difficult_strain_count,
            flashlight_difficulty,
            aim_top_weighted_slider_factor: _,
            speed_top_weighted_slider_factor: _,
            nested_score_per_object: _,
            legacy_score_base_multiplier: _,
            maximum_legacy_combo_score,
        } = diff;

        assert_eq_f64(
            &self.difficulty.stars,
            star_rating.to_native(),
            "star_rating",
        )?;
        assert_eq_u32(
            &self.difficulty.max_combo,
            max_combo.to_native(),
            "max_combo",
        )?;
        assert_eq_f64(
            &self.difficulty.aim,
            aim_difficulty.to_native(),
            "aim_difficulty",
        )?;
        assert_eq_f64(
            &self.difficulty.speed,
            speed_difficulty.to_native(),
            "speed_difficulty",
        )?;
        assert_eq_f64(
            &self.difficulty.speed_note_count,
            speed_note_count.to_native(),
            "speed_note_count",
        )?;
        assert_eq_f64(
            &self.difficulty.aim_difficult_slider_count,
            aim_difficult_slider_count.to_native(),
            "aim_difficult_slider_count",
        )?;
        assert_eq_f64(
            &self.difficulty.aim_difficult_strain_count,
            aim_difficult_strain_count.to_native(),
            "aim_difficult_strain_count",
        )?;
        assert_eq_f64(
            &self.difficulty.speed_difficult_strain_count,
            speed_difficult_strain_count.to_native(),
            "speed_difficult_strain_count",
        )?;
        if let Some(flashlight_difficulty) = flashlight_difficulty.as_ref() {
            assert_eq_f64(
                &self.difficulty.flashlight,
                flashlight_difficulty.to_native(),
                "flashlight_difficulty",
            )?;
        }
        assert_eq_f64(
            &self.difficulty.slider_factor,
            slider_factor.to_native(),
            "slider_factor",
        )?;
        assert_eq_f64(
            &self.difficulty.maximum_legacy_combo_score,
            maximum_legacy_combo_score.to_native(),
            "maximum_legacy_combo_score",
        )?;

        assert_eq_f64(&self.pp, pp.to_native(), "pp")?;
        assert_eq_f64(&self.pp_acc, accuracy.to_native(), "pp_acc")?;
        assert_eq_f64(&self.pp_aim, aim.to_native(), "pp_aim")?;
        assert_eq_f64(&self.pp_flashlight, flashlight.to_native(), "pp_flashlight")?;
        assert_eq_f64(&self.pp_speed, speed.to_native(), "pp_speed")?;
        assert_eq_f64(
            &self.effective_miss_count,
            effective_miss_count.to_native(),
            "effective_miss_count",
        )?;
        assert_eq_option_f64(
            self.speed_deviation.as_ref(),
            speed_deviation.as_ref().map(|s| s.to_native()),
            "speed_deviation",
        )?;

        Ok(())
    }
}

impl AssertEq<TaikoPerformanceAttributes, TaikoDifficultyAttributes>
    for rosu_pp::taiko::TaikoPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &TaikoPerformanceAttributes,
        diff: &TaikoDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let TaikoPerformanceAttributes {
            difficulty,
            accuracy,
            estimated_unstable_rate,
            pp,
        } = perf;

        let TaikoDifficultyAttributes {
            star_rating,
            max_combo,
            rhythm_difficulty,
            mono_stamina_factor,
            consistency_factor,
        } = diff;

        assert_eq_f64(&self.difficulty.stars, *star_rating, "star_rating")?;
        assert_eq_u32(&self.difficulty.max_combo, *max_combo, "max_combo")?;
        assert_eq_f64(
            &self.difficulty.mono_stamina_factor,
            *mono_stamina_factor,
            "mono_stamina_factor",
        )?;
        assert_eq_f64(
            &self.difficulty.rhythm,
            *rhythm_difficulty,
            "rhythm_difficulty",
        )?;
        assert_eq_f64(
            &self.difficulty.consistency_factor,
            *consistency_factor,
            "consistency_factor",
        )?;

        assert_eq_f64(&self.pp_difficulty, *difficulty, "difficulty")?;
        assert_eq_f64(&self.pp_acc, *accuracy, "accuracy")?;
        assert_eq_option_f64(
            self.estimated_unstable_rate.as_ref(),
            estimated_unstable_rate.as_ref().map(|n| *n),
            "estimated_unstable_rate",
        )?;
        assert_eq_f64(&self.pp, *pp, "pp")?;

        Ok(())
    }
}

impl AssertEq<ArchivedTaikoPerformanceAttributes, ArchivedTaikoDifficultyAttributes>
    for rosu_pp::taiko::TaikoPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &ArchivedTaikoPerformanceAttributes,
        diff: &ArchivedTaikoDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let ArchivedTaikoPerformanceAttributes {
            difficulty,
            accuracy,
            estimated_unstable_rate,
            pp,
        } = perf;

        let ArchivedTaikoDifficultyAttributes {
            star_rating,
            max_combo,
            rhythm_difficulty,
            mono_stamina_factor,
            consistency_factor,
        } = diff;

        assert_eq_f64(
            &self.difficulty.stars,
            star_rating.to_native(),
            "star_rating",
        )?;
        assert_eq_u32(
            &self.difficulty.max_combo,
            max_combo.to_native(),
            "max_combo",
        )?;
        assert_eq_f64(
            &self.difficulty.mono_stamina_factor,
            mono_stamina_factor.to_native(),
            "mono_stamina_factor",
        )?;
        assert_eq_f64(
            &self.difficulty.rhythm,
            rhythm_difficulty.to_native(),
            "rhythm_difficulty",
        )?;
        assert_eq_f64(
            &self.difficulty.consistency_factor,
            consistency_factor.to_native(),
            "consistency_factor",
        )?;

        assert_eq_f64(&self.pp_difficulty, difficulty.to_native(), "difficulty")?;
        assert_eq_f64(&self.pp_acc, accuracy.to_native(), "accuracy")?;
        assert_eq_option_f64(
            self.estimated_unstable_rate.as_ref(),
            estimated_unstable_rate.as_ref().map(|n| n.to_native()),
            "estimated_unstable_rate",
        )?;
        assert_eq_f64(&self.pp, pp.to_native(), "pp")?;

        Ok(())
    }
}

impl AssertEq<CatchPerformanceAttributes, CatchDifficultyAttributes>
    for rosu_pp::catch::CatchPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &CatchPerformanceAttributes,
        diff: &CatchDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let CatchPerformanceAttributes { pp } = perf;

        let CatchDifficultyAttributes {
            star_rating,
            max_combo,
        } = diff;

        assert_eq_f64(&self.difficulty.stars, *star_rating, "star_rating")?;
        assert_eq_u32(&self.difficulty.max_combo(), *max_combo, "max_combo")?;

        assert_eq_f64(&self.pp, *pp, "pp")?;

        Ok(())
    }
}

impl AssertEq<ArchivedCatchPerformanceAttributes, ArchivedCatchDifficultyAttributes>
    for rosu_pp::catch::CatchPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &ArchivedCatchPerformanceAttributes,
        diff: &ArchivedCatchDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let ArchivedCatchPerformanceAttributes { pp } = perf;

        let ArchivedCatchDifficultyAttributes {
            star_rating,
            max_combo,
        } = diff;

        assert_eq_f64(
            &self.difficulty.stars,
            star_rating.to_native(),
            "star_rating",
        )?;
        assert_eq_u32(
            &self.difficulty.max_combo(),
            max_combo.to_native(),
            "max_combo",
        )?;

        assert_eq_f64(&self.pp, pp.to_native(), "pp")?;

        Ok(())
    }
}

impl AssertEq<ManiaPerformanceAttributes, ManiaDifficultyAttributes>
    for rosu_pp::mania::ManiaPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &ManiaPerformanceAttributes,
        diff: &ManiaDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let ManiaPerformanceAttributes { difficulty, pp } = perf;

        let ManiaDifficultyAttributes {
            star_rating,
            max_combo,
        } = diff;

        assert_eq_f64(&self.difficulty.stars, *star_rating, "star_rating")?;
        assert_eq_u32(&self.difficulty.max_combo, *max_combo, "max_combo")?;

        assert_eq_f64(&self.pp_difficulty, *difficulty, "difficulty")?;
        assert_eq_f64(&self.pp, *pp, "pp")?;

        Ok(())
    }
}

impl AssertEq<ArchivedManiaPerformanceAttributes, ArchivedManiaDifficultyAttributes>
    for rosu_pp::mania::ManiaPerformanceAttributes
{
    fn assert_eq(
        &self,
        perf: &ArchivedManiaPerformanceAttributes,
        diff: &ArchivedManiaDifficultyAttributes,
    ) -> Result<(), NotEq> {
        let ArchivedManiaPerformanceAttributes { difficulty, pp } = perf;

        let ArchivedManiaDifficultyAttributes {
            star_rating,
            max_combo,
        } = diff;

        assert_eq_f64(
            &self.difficulty.stars,
            star_rating.to_native(),
            "star_rating",
        )?;
        assert_eq_u32(
            &self.difficulty.max_combo,
            max_combo.to_native(),
            "max_combo",
        )?;

        assert_eq_f64(&self.pp_difficulty, difficulty.to_native(), "difficulty")?;
        assert_eq_f64(&self.pp, pp.to_native(), "pp")?;

        Ok(())
    }
}

fn assert_eq_f64(actual: &f64, expected: f64, field: &'static str) -> Result<(), NotEq> {
    if (actual - expected).abs() < f64::EPSILON {
        return Ok(());
    }

    let values = Values::new_f64(expected, *actual);

    Err(NotEq { field, values })
}

fn assert_eq_option_f64(
    actual: Option<&f64>,
    expected: Option<f64>,
    field: &'static str,
) -> Result<(), NotEq> {
    let values = match (actual, expected) {
        (None, None) => return Ok(()),
        (None, expected @ Some(_)) => Values::new_option_f64(expected, None),
        (actual @ Some(_), None) => Values::new_option_f64(None, actual.copied()),
        (Some(actual), Some(expected)) => {
            if (actual - expected).abs() < f64::EPSILON {
                return Ok(());
            }

            Values::new_f64(expected, *actual)
        }
    };

    Err(NotEq { field, values })
}

fn assert_eq_u32(actual: &u32, expected: u32, field: &'static str) -> Result<(), NotEq> {
    if *actual == expected {
        return Ok(());
    }

    let values = Values::new_u32(expected, *actual);

    Err(NotEq { field, values })
}
