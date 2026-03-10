use std::{
    fs::{DirEntry, File},
    mem::MaybeUninit,
    process,
    time::Instant,
};

use rosu_pp::{model::mode::GameMode, Beatmap};

use crate::{
    model::{
        mode::{Catch, IGameMode, Mania, Osu, Taiko},
        object::{ArchivableSimulateObject, SimulateObject},
    },
    util::serializer::Serializer,
    MAP_PATH, PERF_CALC, SIMULATE_OUTPUT,
};

/// Read files from `MAP_PATH` directory, calculate their attributes, and
/// serialize the results to `OUTPUT`
pub fn calculate(overwrite: bool) {
    eprintln!("Calculating...");

    fn file_iter() -> impl Iterator<Item = std::io::Result<DirEntry>> {
        std::fs::read_dir(&*MAP_PATH).unwrap()
    }

    let output = File::options()
        .write(true)
        .append(!overwrite)
        .truncate(overwrite)
        .create(true)
        .open(&*SIMULATE_OUTPUT)
        .unwrap();

    let mut alloc = [MaybeUninit::uninit(); 1 << 15];
    let mut serializer = Serializer::new(output, &mut alloc);

    let count = file_iter().count();
    let mut curr = 0;
    let start = Instant::now();

    for entry in file_iter() {
        curr += 1;

        if curr % 100 == 0 {
            eprintln!(
                "[{elapsed:?}] {curr}/{count} [{osu}/{taiko}/{catch}/{mania}]",
                elapsed = start.elapsed(),
                osu = serializer.osu,
                taiko = serializer.taiko,
                catch = serializer.catch,
                mania = serializer.mania,
            );
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                println!("Failed to read entry: {err}");
                continue;
            }
        };

        let path = entry.path();

        let Some(path) = path.to_str() else {
            return println!("Invalid path `{}`", path.display());
        };

        let map = match Beatmap::from_path(path) {
            Ok(map) => map,
            Err(err) => {
                println!("Failed to decode map {path}: {err}");
                continue;
            }
        };

        match map.mode {
            GameMode::Osu => {
                simulate::<Osu>(path, &mut serializer);
                simulate::<Taiko>(path, &mut serializer);
                simulate::<Catch>(path, &mut serializer);
                simulate::<Mania>(path, &mut serializer);
            }
            GameMode::Taiko => simulate::<Taiko>(path, &mut serializer),
            GameMode::Catch => simulate::<Catch>(path, &mut serializer),
            GameMode::Mania => simulate::<Mania>(path, &mut serializer),
        }
    }

    println!("Done calculating");
}

/// Execute PerformanceCalculator.dll, deserialize its output, and re-serialize it via rkyv
fn simulate<M>(path: &str, serializer: &mut Serializer)
where
    M: IGameMode,
    ArchivableSimulateObject: From<SimulateObject<M>>,
{
    fn parse_map_id(path: &str) -> Option<i32> {
        if let Some(Ok(map_id)) = path
            .trim_end_matches(".osu")
            .rsplit('/')
            .next()
            .map(str::parse)
        {
            Some(map_id)
        } else {
            println!("Missing map id in path `{path}`");

            None
        }
    }

    fn execute_command(path: &str, mode: &str, all_mods: &[&str]) -> Option<process::Output> {
        if all_mods.is_empty() {
            return None;
        }

        let mut cmd = process::Command::new("dotnet");
        let args = [&*PERF_CALC, "simulate", mode, path, "--json"];
        cmd.args(args);

        for mods in all_mods {
            cmd.args(["--mod", mods]);
        }

        let output = match cmd.output() {
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

    let Some(map_id) = parse_map_id(path) else {
        return;
    };

    let Some(output) = execute_command(path, M::ARG, M::MODS) else {
        return;
    };

    match serde_json::from_slice::<Vec<SimulateObject<M>>>(&output.stdout) {
        Ok(simulate_objects) => {
            for mut simulate_object in simulate_objects {
                simulate_object.score.beatmap_id = map_id;
                let obj = ArchivableSimulateObject::from(simulate_object);
                serializer.serialize(&obj);
                serializer.increment_mode(&obj);
            }
        }
        Err(err) => {
            println!("Failed to deserialize objects: ");

            match std::str::from_utf8(&output.stdout) {
                Ok(stdout) => println!("`{stdout}`: {err}"),
                Err(_) => println!("Invalid UTF-8"),
            }
        }
    }
}
