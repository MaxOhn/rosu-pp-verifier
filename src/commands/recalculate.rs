use std::{fs, process};

use memmap2::MmapMut;
use rkyv::{option::ArchivedOption, rancor::BoxedError, seal::Seal, Archived};

use crate::{
    model::{
        difficulty::{
            ArchivedCatchDifficultyAttributes, ArchivedManiaDifficultyAttributes,
            ArchivedOsuDifficultyAttributes, ArchivedTaikoDifficultyAttributes,
        },
        mode::{Catch, Mania, Osu, Taiko},
        object::{
            ArchivableSimulateObject, ArchivedSimulateAttributes, ArchivedSimulateObject,
            SimulateAttributes, SimulateObject,
        },
        performance::{
            ArchivedCatchPerformanceAttributes, ArchivedManiaPerformanceAttributes,
            ArchivedOsuPerformanceAttributes, ArchivedTaikoPerformanceAttributes,
        },
    },
    MAP_PATH, MAX_ALIGN, PERF_CALC, SEPARATOR, SIMULATE_OUTPUT,
};

/// Finds attribute entries in `OUTPUT` returning true for `should_recalc`,
/// recalculates them via `PERF_CALC`, and mutates `OUTPUT`.
pub fn recalculate() {
    if !cfg!(debug_assertions) {
        panic!("Do not recalculate scores on release mode");
    }

    eprintln!("Recalculating...");

    let mut input = match fs::File::options()
        .read(true)
        .write(true)
        .open(&*SIMULATE_OUTPUT)
    {
        Ok(input) => match unsafe { MmapMut::map_mut(&input) } {
            Ok(mmap) => mmap,
            Err(err) => return println!("Failed to mmap input: {err}"),
        },
        Err(err) => return println!("Failed to open input: {err}"),
    };

    let mut start = 0;
    let mut mods = String::new();

    loop {
        let mut end = start + 1;

        while end < input.len() && !input[end..].starts_with(SEPARATOR) {
            end += 1;
        }

        if end >= input.len() {
            break;
        }

        end += SEPARATOR.len();

        const_assert!(SEPARATOR.len() <= MAX_ALIGN);
        let bytes = &mut input[start..end - MAX_ALIGN];

        let simulate_object =
            match rkyv::access_mut::<Archived<ArchivableSimulateObject>, BoxedError>(bytes) {
                Ok(obj) => obj,
                Err(err) => {
                    println!("Failed to read archive `{bytes:?}`: {err}");
                    start = end;

                    continue;
                }
            };

        let map_id = simulate_object.beatmap_id;

        if !should_recalc(&simulate_object) {
            start = end;
            continue;
        }

        let mode = simulate_object.mode();

        mods.clear();

        for m in simulate_object.mods.acronyms() {
            mods.push_str(m);
        }

        if mods.is_empty() {
            mods.push_str("NM");
        }

        simulate(simulate_object, &mods);
        println!("Recalculated {map_id} {mode} {mods}");

        start = end;
    }

    println!("Done recalculating");
}

fn should_recalc(simulate_object: &Seal<'_, ArchivedSimulateObject>) -> bool {
    const RECALC_MAP_IDS: &[Archived<i32>] = &[];

    RECALC_MAP_IDS.contains(&simulate_object.beatmap_id)
}

fn simulate(simulate_object: Seal<'_, ArchivedSimulateObject>, mods: &str) {
    fn execute_command(path: &str, mode: &str, mods: &str) -> Option<process::Output> {
        let mut cmd = process::Command::new("dotnet");
        let args = [
            &*PERF_CALC,
            "simulate",
            mode,
            path,
            "--json",
            "--no-classic",
            "--mod",
            mods,
        ];

        let output = match cmd.args(args).output() {
            Ok(output) => output,
            Err(err) => {
                println!("Failed to execute command `{cmd:?}`: {err}");

                return None;
            }
        };

        if output.status.success() {
            return Some(output);
        }

        println!("Error for path `{path}`:");

        match std::str::from_utf8(&output.stderr) {
            Ok(stderr) => println!("{stderr}"),
            Err(_) => println!("Invalid UTF-8"),
        }

        None
    }

    let path = format!("{}{}.osu", &*MAP_PATH, simulate_object.beatmap_id);
    let mode = simulate_object.mode();

    let Some(output) = execute_command(&path, mode, mods) else {
        return;
    };

    let parsed = match simulate_object.attrs {
        ArchivedSimulateAttributes::Osu { .. } => {
            serde_json::from_slice::<Vec<SimulateObject<Osu>>>(&output.stdout)
                .map(|vec| vec.into_iter().map(ArchivableSimulateObject::from).next())
        }
        ArchivedSimulateAttributes::Taiko { .. } => {
            serde_json::from_slice::<Vec<SimulateObject<Taiko>>>(&output.stdout)
                .map(|vec| vec.into_iter().map(ArchivableSimulateObject::from).next())
        }
        ArchivedSimulateAttributes::Catch { .. } => {
            serde_json::from_slice::<Vec<SimulateObject<Catch>>>(&output.stdout)
                .map(|vec| vec.into_iter().map(ArchivableSimulateObject::from).next())
        }
        ArchivedSimulateAttributes::Mania { .. } => {
            serde_json::from_slice::<Vec<SimulateObject<Mania>>>(&output.stdout)
                .map(|vec| vec.into_iter().map(ArchivableSimulateObject::from).next())
        }
    };

    match parsed {
        Ok(Some(new)) => modify(simulate_object, new),
        Err(err) => {
            println!("Failed to deserialize objects: ");

            match std::str::from_utf8(&output.stdout) {
                Ok(stdout) => println!("`{stdout}`: {err}"),
                Err(_) => println!("Invalid UTF-8"),
            }
        }
        Ok(None) => println!("No object was deserialized"),
    }
}

fn modify(old: Seal<'_, ArchivedSimulateObject>, new: ArchivableSimulateObject) {
    // SAFETY: Fields won't be moved, pinky promise
    let old = unsafe { old.unseal_unchecked() };

    match &mut old.attrs {
        ArchivedSimulateAttributes::Osu {
            perf: old_perf,
            diff: old_diff,
        } => {
            let SimulateAttributes::Osu { perf, diff } = new.attrs else {
                return println!("Invalid mode");
            };

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
            } = old_perf;

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
            } = old_diff;

            *aim = perf.aim.into();
            *speed = perf.speed.into();
            *accuracy = perf.accuracy.into();
            *flashlight = perf.flashlight.into();
            *effective_miss_count = perf.effective_miss_count.into();

            if let Some(mut old_speed_deviation) =
                ArchivedOption::as_seal(Seal::new(speed_deviation))
            {
                if let Some(speed_deviation) = perf.speed_deviation {
                    *old_speed_deviation = speed_deviation.into();
                }
            }

            *pp = perf.pp.into();

            *star_rating = diff.star_rating.into();
            *max_combo = diff.max_combo.into();
            *aim_difficulty = diff.aim_difficulty.into();
            *aim_difficult_slider_count = diff.aim_difficult_slider_count.into();
            *aim_difficult_strain_count = diff.aim_difficult_strain_count.into();
            *speed_difficulty = diff.speed_difficulty.into();
            *speed_note_count = diff.speed_note_count.into();
            *speed_difficult_strain_count = diff.speed_difficult_strain_count.into();
            *flashlight_difficulty = diff
                .flashlight_difficulty
                .map(From::from)
                .map_or(ArchivedOption::None, ArchivedOption::Some);
            *slider_factor = diff.slider_factor.into();
            *maximum_legacy_combo_score = diff.maximum_legacy_combo_score.into();
        }
        ArchivedSimulateAttributes::Taiko {
            perf: old_perf,
            diff: old_diff,
        } => {
            let SimulateAttributes::Taiko { perf, diff } = new.attrs else {
                return println!("Invalid mode");
            };

            let ArchivedTaikoPerformanceAttributes {
                difficulty,
                accuracy,
                estimated_unstable_rate,
                pp,
            } = old_perf;

            let ArchivedTaikoDifficultyAttributes {
                star_rating,
                max_combo,
                rhythm_difficulty,
                mono_stamina_factor,
                consistency_factor,
            } = old_diff;

            *difficulty = perf.difficulty.into();
            *accuracy = perf.accuracy.into();

            if let Some(mut old_estimated_unstable_rate) =
                ArchivedOption::as_seal(Seal::new(estimated_unstable_rate))
            {
                if let Some(estimated_unstable_rate) = perf.estimated_unstable_rate {
                    *old_estimated_unstable_rate = estimated_unstable_rate.into();
                }
            }

            *pp = perf.pp.into();

            *star_rating = diff.star_rating.into();
            *max_combo = diff.max_combo.into();
            *rhythm_difficulty = diff.rhythm_difficulty.into();
            *mono_stamina_factor = diff.mono_stamina_factor.into();
            *consistency_factor = diff.consistency_factor.into();
        }
        ArchivedSimulateAttributes::Catch {
            perf: old_perf,
            diff: old_diff,
        } => {
            let SimulateAttributes::Catch { perf, diff } = new.attrs else {
                return println!("Invalid mode");
            };

            let ArchivedCatchPerformanceAttributes { pp } = old_perf;

            let ArchivedCatchDifficultyAttributes {
                star_rating,
                max_combo,
            } = old_diff;

            *pp = perf.pp.into();

            *star_rating = diff.star_rating.into();
            *max_combo = diff.max_combo.into();
        }
        ArchivedSimulateAttributes::Mania {
            perf: old_perf,
            diff: old_diff,
        } => {
            let SimulateAttributes::Mania { perf, diff } = new.attrs else {
                return println!("Invalid mode");
            };

            let ArchivedManiaPerformanceAttributes { difficulty, pp } = old_perf;

            let ArchivedManiaDifficultyAttributes {
                star_rating,
                max_combo,
            } = old_diff;

            *difficulty = perf.difficulty.into();
            *pp = perf.pp.into();

            *star_rating = diff.star_rating.into();
            *max_combo = diff.max_combo.into();
        }
    }
}
