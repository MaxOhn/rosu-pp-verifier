use std::{
    fmt::Write,
    fs::File,
    ops::ControlFlow,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use memmap2::{Mmap, MmapMut};
use rand::seq::SliceRandom;
use rkyv::{
    munge::munge,
    rancor::{Panic, ResultExt},
    seal::Seal,
    vec::ArchivedVec,
};
use rosu_mods::GameMode;
use rosu_pp::{
    any::PerformanceAttributes, catch::CatchPerformance, mania::ManiaPerformance,
    osu::OsuPerformance, taiko::TaikoPerformance, Beatmap,
};

use crate::{
    model::{
        data_score::ArchivedDataScore,
        mode::{Catch, Mania, Osu, Taiko},
        mods::ArchivedGameModSettingValue,
        object::{ArchivableSimulateObject, SimulateAttributes, SimulateObject},
    },
    util::assert_eq::{AssertEq, NotEq},
    LOAD_OUTPUT, MAP_PATH, PERF_CALC,
};

use self::progress::ScoresProgress;

struct Done {
    count: usize,
    total: usize,
}

impl Done {
    fn new() -> Self {
        let input = match File::options().read(true).open(&*LOAD_OUTPUT) {
            Ok(input) => match unsafe { Mmap::map(&input) } {
                Ok(mmap) => mmap,
                Err(err) => panic!("Failed to mmap input: {err}"),
            },
            Err(err) => panic!("Failed to open input: {err}"),
        };

        let scores = rkyv::access::<ArchivedVec<ArchivedDataScore>, Panic>(&input).always_ok();
        let done = scores.iter().filter(|score| score.checked).count();

        Self {
            count: done,
            total: scores.len(),
        }
    }
}

pub fn loaded(minutes: Option<u64>) {
    let Done { count, total } = Done::new();

    eprintln!(
        "Done: {count}/{total} ({:.4})%",
        ((count * 100) as f64 / total as f64)
    );

    let left = total - count;

    if !cfg!(debug_assertions) {
        panic!("Do not compare scores on release mode");
    }

    let mut input = match File::options().read(true).write(true).open(&*LOAD_OUTPUT) {
        Ok(input) => match unsafe { MmapMut::map_mut(&input) } {
            Ok(mmap) => mmap,
            Err(err) => return println!("Failed to mmap input: {err}"),
        },
        Err(err) => return println!("Failed to open input: {err}"),
    };

    let scores = rkyv::access_mut::<ArchivedVec<ArchivedDataScore>, Panic>(&mut input).always_ok();

    let mut indices = (0..scores.len() as u32).collect::<Vec<_>>();
    indices.shuffle(&mut rand::thread_rng());

    if let Some(minutes) = minutes {
        let start = Instant::now();

        let duration = Duration::from_secs(minutes * 60);
        let scores_progress = ScoresProgress::new(left);
        let template = format!("{{scores}} {{wide_bar}} {{elapsed}}/{duration:?}");
        let style = ProgressStyle::with_template(&template)
            .unwrap()
            .with_key("scores", scores_progress.clone());
        let progress = ProgressBar::with_draw_target(
            Some(duration.as_millis() as u64),
            ProgressDrawTarget::stderr(),
        )
        .with_style(style);

        let mut since_last = start.elapsed();

        iterate_scores(
            ArchivedVec::as_slice_seal(scores),
            &indices,
            &progress,
            || {
                let elapsed = start.elapsed();

                if elapsed >= duration {
                    return ControlFlow::Break(());
                }

                progress.inc((elapsed - since_last).as_millis() as u64);
                since_last = elapsed;

                ControlFlow::Continue(())
            },
            || scores_progress.inc(),
        );

        progress.finish();
    } else {
        let template = "Scores: {pos}/{len} | ETA: {eta} {wide_bar} {elapsed}";
        let style = ProgressStyle::with_template(template).unwrap();
        let progress =
            ProgressBar::with_draw_target(Some(left as u64), ProgressDrawTarget::stderr())
                .with_style(style);

        iterate_scores(
            ArchivedVec::as_slice_seal(scores),
            &indices,
            &progress,
            || ControlFlow::Continue(()),
            || progress.inc(1),
        );

        progress.finish();
    }
}

fn iterate_scores<B, A>(
    mut scores: Seal<'_, [ArchivedDataScore]>,
    indices: &[u32],
    progress: &ProgressBar,
    mut before: B,
    mut increment: A,
) where
    B: FnMut() -> ControlFlow<(), ()>,
    A: FnMut(),
{
    for i in indices {
        let score = scores.as_mut().index(*i as usize);

        if let ControlFlow::Break(()) = before() {
            return;
        }

        if calculate_score(score, progress) {
            increment();
        }
    }
}

/// Returns whether the progress bar should be incremented.
fn calculate_score(score: Seal<'_, ArchivedDataScore>, progress: &ProgressBar) -> bool {
    if score.checked {
        return false;
    } else if score.data.mods.varying_clock_rate() {
        munge!(let ArchivedDataScore { mut checked, .. } = score);
        *checked = true;

        // rosu-pp does not handle varying clock rate
        return true;
    } else if score.data.mods.contains_random() && score.mode == 0 {
        // rosu-pp does not handle the random mod in osu!standard
        return false;
    } else if score.mode == 0 && score.lazer && score.data.mods.contains_classic() {
        // osu-tools doesn't handle CL lazer scores correctly
        return false;
    }

    let map_path = format!("{}{}.osu", &*MAP_PATH, score.map_id);

    let stats = &score.data.stats;
    let max_stats = &score.data.max_stats;
    let lazer = score.lazer;

    let mode = match score.mode {
        0 => GameMode::Osu,
        1 => GameMode::Taiko,
        2 => GameMode::Catch,
        3 => GameMode::Mania,
        other => {
            println!("Invalid mode {other}");

            return false;
        }
    };

    let mode_str = match mode {
        GameMode::Osu => "osu",
        GameMode::Taiko => "taiko",
        GameMode::Catch => "catch",
        GameMode::Mania => "mania",
    };

    let mut cmd = Command::new("dotnet");

    cmd.args([
        &*PERF_CALC,
        "simulate",
        mode_str,
        &map_path,
        "--json",
        "--accuracy",
        &(score.acc * 100.0).to_string(),
    ]);

    let mut no_slider_head_acc = !lazer;

    for gamemod in score.data.mods.iter() {
        let acronym = gamemod.acronym.as_ref();
        cmd.args(["--mod", acronym]);

        if acronym == "CL" {
            no_slider_head_acc = true;
        }

        for setting in gamemod.settings.iter() {
            let value = setting.value.as_str();

            if setting.key.as_ref() == "no_slider_head_accuracy" {
                if let ArchivedGameModSettingValue::Bool(value) = setting.value {
                    no_slider_head_acc = value;
                }
            }

            let value = format!("{}_{}={value}", gamemod.acronym, setting.key,);
            cmd.args(["--mod-option", &value]);
        }
    }

    match mode {
        GameMode::Osu => {
            cmd.args([
                "--combo",
                &score.combo.to_string(),
                "--mehs",
                &stats.meh.to_string(),
                "--goods",
                &stats.ok.to_string(),
                "--misses",
                &stats.miss.to_string(),
            ]);

            if lazer {
                cmd.args([
                    "--large-tick-misses",
                    &(max_stats.large_tick_hit - stats.large_tick_hit).to_string(),
                ]);

                if no_slider_head_acc {
                    cmd.args([
                        "--slider-tail-misses",
                        &(max_stats.small_tick_hit - stats.small_tick_hit).to_string(),
                    ]);
                } else {
                    cmd.args([
                        "--slider-tail-misses",
                        &(max_stats.slider_tail_hit - stats.slider_tail_hit).to_string(),
                    ]);
                }
            }
        }
        GameMode::Taiko => {
            cmd.args([
                "--goods",
                &stats.ok.to_string(),
                "--misses",
                &stats.miss.to_string(),
                "--combo",
                &score.combo.to_string(),
            ]);
        }
        GameMode::Catch => {
            cmd.args([
                "--tiny-droplets",
                &stats.small_tick_hit.to_string(),
                "--droplets",
                &stats.large_tick_hit.to_string(),
                "--misses",
                &stats.miss.to_string(),
                "--combo",
                &score.combo.to_string(),
            ]);
        }
        GameMode::Mania => {
            cmd.args([
                "--mehs",
                &stats.meh.to_string(),
                "--goods",
                &stats.good.to_string(),
                "--oks",
                &stats.ok.to_string(),
                "--greats",
                &stats.great.to_string(),
                "--misses",
                &stats.miss.to_string(),
            ]);
        }
    }

    let child = match cmd.stdout(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(err) => {
            println!("Failed to spawn child: {err}");

            return false;
        }
    };

    let map = match Beatmap::from_path(&map_path) {
        Ok(map) => map,
        Err(err) => {
            println!("Failed to open map at `{map_path}`: {err}");

            return false;
        }
    };

    if let Err(err) = map.check_suspicion() {
        progress.suspend(|| println!("[SUSPICIOUS] map_id={} ({err})", score.map_id));
    }

    let mods = match score.data.mods.convert(mode) {
        Ok(mods) => mods,
        Err(err) => {
            println!("Failed to convert mods: {err}");

            return false;
        }
    };

    let mut rosu_str = String::new();

    macro_rules! rosu_calc {
        (
            $perf:ident ::new($map:ident)
                $( .$fn:ident($arg:expr) )*
        ) => {{
            rosu_str.push_str(
                concat!(stringify!($perf), "::new(", stringify!($map), ")")
            );

            $( write!(rosu_str, concat!(".", stringify!($fn), "({})"), $arg).unwrap(); )*

            rosu_str.push_str(".calculate()");

            $perf::new($map) $( .$fn($arg) )*
        }}
    }

    let attrs = match mode {
        GameMode::Osu => {
            let Ok(attrs) = rosu_calc!(OsuPerformance::new(map)
                .lazer(lazer)
                .mods(mods)
                .combo(score.combo.to_native())
                .n300(stats.great.to_native())
                .n100(stats.ok.to_native())
                .n50(stats.meh.to_native())
                .misses(stats.miss.to_native())
                .slider_end_hits(if lazer && no_slider_head_acc {
                    stats.small_tick_hit.to_native()
                } else {
                    stats.slider_tail_hit.to_native()
                })
                .large_tick_hits(stats.large_tick_hit.to_native()))
            .calculate() else {
                println!("Failed to convert map at `{map_path}` to Osu");

                return false;
            };

            PerformanceAttributes::Osu(attrs)
        }
        GameMode::Taiko => {
            let Ok(attrs) = rosu_calc!(TaikoPerformance::new(map)
                .mods(mods)
                .combo(score.combo.to_native())
                .n300(stats.great.to_native())
                .n100(stats.ok.to_native())
                .misses(stats.miss.to_native()))
            .calculate() else {
                println!("Failed to convert map at `{map_path}` to Taiko");

                return false;
            };

            PerformanceAttributes::Taiko(attrs)
        }
        GameMode::Catch => {
            let Ok(attrs) = rosu_calc!(CatchPerformance::new(map)
                .mods(mods)
                .combo(score.combo.to_native())
                .fruits(stats.great.to_native())
                .droplets(stats.large_tick_hit.to_native())
                .tiny_droplets(stats.small_tick_hit.to_native())
                .misses(stats.miss.to_native()))
            .calculate() else {
                println!("Failed to convert map at `{map_path}` to Catch");

                return false;
            };

            PerformanceAttributes::Catch(attrs)
        }
        GameMode::Mania => {
            let Ok(attrs) = rosu_calc!(ManiaPerformance::new(map)
                .mods(mods)
                .lazer(lazer)
                .n320(stats.perfect.to_native())
                .n300(stats.great.to_native())
                .n200(stats.good.to_native())
                .n100(stats.ok.to_native())
                .n50(stats.meh.to_native())
                .misses(stats.miss.to_native()))
            .calculate() else {
                println!("Failed to convert map at `{map_path}` to Mania");

                return false;
            };

            PerformanceAttributes::Mania(attrs)
        }
    };

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(err) => {
            println!("Failed to wait on child: {err}");

            return false;
        }
    };

    if !output.status.success() {
        let err = std::str::from_utf8(&output.stderr).unwrap_or("invalid UTF-8");
        println!("Error for path `{map_path}`: {err}");

        return false;
    }

    let res = match mode {
        GameMode::Osu => serde_json::from_slice::<SimulateObject<Osu>>(&output.stdout)
            .map(ArchivableSimulateObject::from),
        GameMode::Taiko => serde_json::from_slice::<SimulateObject<Taiko>>(&output.stdout)
            .map(ArchivableSimulateObject::from),
        GameMode::Catch => serde_json::from_slice::<SimulateObject<Catch>>(&output.stdout)
            .map(ArchivableSimulateObject::from),
        GameMode::Mania => serde_json::from_slice::<SimulateObject<Mania>>(&output.stdout)
            .map(ArchivableSimulateObject::from),
    };

    match res {
        Ok(obj) => {
            if !obj.statistics.is_eq(stats, score.mode, lazer) {
                let score_ref = score.as_ref();

                if !skip_ignore(score_ref) {
                    progress.suspend(|| {
                        println!(
                            "[IGNORE] {score_ref:?} | Simulated: {:?} | {} | {rosu_str}",
                            obj.statistics,
                            cmd_str(&cmd),
                        )
                    });
                }

                return false;
            }

            if let Err(not_eq) = assert_eq(&obj, attrs) {
                progress.suspend(|| {
                    println!(
                        "[MISMATCH] {:?} {not_eq} | {} | {rosu_str}",
                        score.unseal_ref(),
                        cmd_str(&cmd),
                    );
                });

                false
            } else {
                munge!(let ArchivedDataScore { mut checked, .. } = score);
                *checked = true;

                true
            }
        }
        Err(err) => {
            println!("Failed to deserialize objects: ");

            match std::str::from_utf8(&output.stdout) {
                Ok(stdout) => println!("`{stdout}`: {err}"),
                Err(_) => println!("Invalid UTF-8"),
            }

            false
        }
    }
}

fn assert_eq(
    data_object: &ArchivableSimulateObject,
    attrs: PerformanceAttributes,
) -> Result<(), NotEq> {
    match (&data_object.attrs, attrs) {
        (SimulateAttributes::Osu { perf, diff }, PerformanceAttributes::Osu(attrs)) => {
            attrs.assert_eq(perf, diff)
        }
        (SimulateAttributes::Taiko { perf, diff }, PerformanceAttributes::Taiko(attrs)) => {
            attrs.assert_eq(perf, diff)
        }
        (SimulateAttributes::Catch { perf, diff }, PerformanceAttributes::Catch(attrs)) => {
            attrs.assert_eq(perf, diff)
        }
        (SimulateAttributes::Mania { perf, diff }, PerformanceAttributes::Mania(attrs)) => {
            attrs.assert_eq(perf, diff)
        }
        _ => panic!("modes dont coincide"),
    }
}

fn cmd_str(cmd: &Command) -> String {
    [cmd.get_program().to_str().unwrap_or("<bad OsStr>"), " "]
        .into_iter()
        .chain(
            cmd.get_args()
                .flat_map(|s| [s.to_str().unwrap_or("<bad OsStr>"), " "]),
        )
        .collect::<String>()
}

fn skip_ignore(score: &ArchivedDataScore) -> bool {
    const IGNORE_STD: &[u32] = &[
        1029976, // sky_delta - Grenade
        1267365, // Camellia - crystallized
        1981090, // Culprate - Acid Rain
        2052199, // Culprate - Acid Rain
        2055234, // Culprate - Acid Rain
        2087153, // Culprate - Acid Rain
        2571858, // RiraN - Unshakable
        2573161, // Frums - XNOR XNOR XNOR
        2573164, // Frums - XNOR XNOR XNOR
    ];

    const IGNORE_TKO: &[u32] = &[];
    const IGNORE_CTB: &[u32] = &[];
    const IGNORE_MNA: &[u32] = &[];

    match score.mode {
        0 => IGNORE_STD.contains(&score.map_id.to_native()),
        1 => IGNORE_TKO.contains(&score.map_id.to_native()),
        2 => IGNORE_CTB.contains(&score.map_id.to_native()),
        3 => IGNORE_MNA.contains(&score.map_id.to_native()),
        _ => false,
    }
}

mod progress {
    use std::{
        fmt::Write,
        sync::{Arc, Mutex},
        time::Instant,
    };

    use indicatif::{style::ProgressTracker, ProgressState};

    #[derive(Clone)]
    pub struct ScoresProgress {
        repr: Arc<Mutex<Repr>>,
    }

    struct Repr {
        curr: usize,
        total: usize,
    }

    impl ScoresProgress {
        pub fn new(total: usize) -> Self {
            Self {
                repr: Arc::new(Mutex::new(Repr { curr: 0, total })),
            }
        }

        pub fn inc(&self) {
            self.repr.lock().unwrap().curr += 1;
        }
    }

    impl ProgressTracker for ScoresProgress {
        fn clone_box(&self) -> Box<dyn ProgressTracker> {
            Box::new(self.to_owned())
        }

        fn tick(&mut self, _: &ProgressState, _: Instant) {}

        fn reset(&mut self, _: &ProgressState, _: Instant) {
            self.repr.lock().unwrap().curr = 0;
        }

        fn write(&self, _: &ProgressState, w: &mut dyn Write) {
            let guard = self.repr.lock().unwrap();
            write!(w, "Scores: {}/{}", guard.curr, guard.total).unwrap();
        }
    }
}
