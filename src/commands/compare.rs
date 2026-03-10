use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::{Duration, Instant},
};

use rkyv::{rancor::BoxedError, Archived};
use rosu_pp::{
    catch::CatchPerformance, mania::ManiaPerformance, osu::OsuPerformance, taiko::TaikoPerformance,
    Beatmap,
};

use crate::{
    model::{
        object::{ArchivableSimulateObject, ArchivedSimulateAttributes, ArchivedSimulateObject},
        recent_map::RecentBeatmap,
    },
    util::assert_eq::{AssertEq, NotEq},
    MAP_PATH, MAX_ALIGN, SEPARATOR, SIMULATE_OUTPUT,
};

/// Read all entries from `OUTPUT`, deserialize them, calculate corresponding
/// rosu-pp attributes, and assert that they are equal
pub fn compare() {
    if !cfg!(debug_assertions) {
        panic!("Do not compare on release mode");
    }

    eprintln!("Comparing...");

    let mut last_print = Instant::now();
    let start = Instant::now();

    let mut input = match File::open(&*SIMULATE_OUTPUT) {
        Ok(input) => BufReader::new(input),
        Err(err) => return println!("Failed to open input: {err}"),
    };

    let mut buf = Vec::with_capacity(256);
    let mut map = RecentBeatmap::default();

    loop {
        buf.clear();

        match input.read_until(SEPARATOR[0], &mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => return println!("Failed to read until separator start: {err}"),
        }

        while !buf.ends_with(SEPARATOR) {
            match input.read_until(SEPARATOR[SEPARATOR.len() - 1], &mut buf) {
                Ok(0) => return println!("Missing separator end"),
                Ok(_) => {}
                Err(err) => return println!("Failed to read until separator end: {err}"),
            }
        }

        const_assert!(SEPARATOR.len() <= MAX_ALIGN);
        let bytes = &buf[..buf.len() - MAX_ALIGN];

        match rkyv::access::<Archived<ArchivableSimulateObject>, BoxedError>(bytes) {
            Ok(simulate_object) => {
                let Some(map) = map.get(simulate_object.beatmap_id) else {
                    continue;
                };

                if let Err(not_eq) = assert_eq(simulate_object, map) {
                    println!(
                        "map_id={map_id} mode={mode} mods={mods:?} {not_eq}",
                        map_id = simulate_object.beatmap_id,
                        mode = simulate_object.mode(),
                        mods = simulate_object.mods,
                    );
                }

                if last_print.elapsed() > Duration::from_secs(60 * 10) {
                    eprintln!("[{:.2?}] {}", start.elapsed(), simulate_object.beatmap_id);
                    last_print = Instant::now();
                }
            }
            Err(err) => println!("Failed to read archive `{bytes:?}`: {err}"),
        }
    }

    println!("[{:.2?}] Done comparing", start.elapsed());
}

fn assert_eq(simulate_object: &ArchivedSimulateObject, map: &Beatmap) -> Result<(), NotEq> {
    let path = format!("{}{}.osu", &*MAP_PATH, simulate_object.beatmap_id);

    macro_rules! match_attrs {
        ( $(
            $variant:ident, $lowercase:ident, $performance:ident;
        )*) => {
            match &simulate_object.attrs {
                $(
                    ArchivedSimulateAttributes::$variant { perf, diff } => {
                        let mut calc = $performance::from(map)
                            .mods(simulate_object.mods.bits());

                        calc = match_attrs!(@LAZER $variant calc);

                        match calc.calculate() {
                            Ok(attrs) => attrs.assert_eq(perf, diff),
                            Err(_) => {
                                println!(
                                    concat!(
                                        "Failed to convert ",
                                        stringify!($lowercase),
                                        " map at `{path}`"
                                    ),
                                    path = path
                                );

                                Ok(())
                            }
                        }
                    }
                )*
            }
        };
        (@LAZER Osu $calc:ident) => {
            $calc.lazer(true)
        };
        (@LAZER $mode:ident $calc:ident) => {
            $calc
        };
    }

    match_attrs! {
        Osu, osu, OsuPerformance;
        Taiko, taiko, TaikoPerformance;
        Catch, catch, CatchPerformance;
        Mania, mania, ManiaPerformance;
    }
}
