use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    time::Instant,
};

use csv::{ReaderBuilder, StringRecord};
use rkyv::{
    rancor::{Panic, ResultExt},
    ser::writer::IoWriter,
};

use crate::{model::data_score::DataScore, LOAD_OUTPUT};

const INSERT_INTO: &str = "INSERT INTO `scores` VALUES ";

macro_rules! prepare_line {
    ( $line:ident ) => {
        match $line
            .strip_prefix(INSERT_INTO)
            .and_then(|s| s.strip_prefix('('))
            .and_then(|s| s.strip_suffix(");\n"))
        {
            Some(scores) => scores,
            None => return println!("Empty data"),
        }
    };
}

pub fn load(mode: Option<u8>) {
    let start = Instant::now();

    let mut output = File::options()
        .write(true)
        .append(false)
        .truncate(true)
        .create(true)
        .open(&*LOAD_OUTPUT)
        .unwrap();

    let mut list = Vec::new();

    match mode {
        None => {
            eprintln!("Loading osu...");
            load_mode(&env::var("SCORES_SQL_OSU").unwrap(), &mut list);
            eprintln!("Loading taiko...");
            load_mode(&env::var("SCORES_SQL_TAIKO").unwrap(), &mut list);
            eprintln!("Loading catch...");
            load_mode(&env::var("SCORES_SQL_CATCH").unwrap(), &mut list);
            eprintln!("Loading mania...");
            load_mode(&env::var("SCORES_SQL_MANIA").unwrap(), &mut list);
        }
        Some(0) => {
            println!("Loading osu...");
            load_mode(&env::var("SCORES_SQL_OSU").unwrap(), &mut list);
        }
        Some(1) => {
            println!("Loading taiko...");
            load_mode(&env::var("SCORES_SQL_TAIKO").unwrap(), &mut list);
        }
        Some(2) => {
            println!("Loading catch...");
            load_mode(&env::var("SCORES_SQL_CATCH").unwrap(), &mut list);
        }
        Some(3) => {
            println!("Loading mania...");
            load_mode(&env::var("SCORES_SQL_MANIA").unwrap(), &mut list);
        }
        Some(_) => return println!("Invalid mode"),
    }

    let load_elapsed = start.elapsed();
    println!("Loaded {} scores in {load_elapsed:?}", list.len());

    let mut writer = IoWriter::new(BufWriter::new(&mut output));
    rkyv::api::high::to_bytes_in::<_, Panic>(&list, &mut writer).always_ok();
    writer.into_inner().flush().unwrap();

    let serialize_elapsed = start.elapsed();
    println!(
        "Serialized scores in {:?}",
        serialize_elapsed - load_elapsed
    );
}

fn load_mode(path: &str, list: &mut Vec<DataScore>) {
    let mut input = match File::open(path) {
        Ok(input) => BufReader::new(input),
        Err(err) => return println!("Failed to open input: {err}"),
    };

    let mut line = String::new();
    let mut record = StringRecord::new();

    skip_to_start(&mut input, &mut line, INSERT_INTO);

    let scores = prepare_line!(line);
    process_scores(scores, &mut record, list);

    loop {
        match input.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => return println!("Failed to read line: {err}"),
        }

        if !line.starts_with(INSERT_INTO) {
            break;
        }

        let scores = prepare_line!(line);
        process_scores(scores, &mut record, list);

        line.clear();
    }
}

fn process_scores(scores: &str, record: &mut StringRecord, list: &mut Vec<DataScore>) {
    for score in scores.split("),(") {
        match ReaderBuilder::new()
            .has_headers(false)
            .quote(b'\'')
            .from_reader(score.as_bytes())
            .read_record(record)
        {
            Ok(true) => {}
            Ok(false) => return println!("No record for score `{score}`"),
            Err(err) => return println!("Record error for score `{score}`: {err}"),
        }

        let score = match record.deserialize::<DataScore>(None) {
            Ok(score) => score,
            Err(err) => return println!("Failed to deserialize score `{score}`\n{err}"),
        };

        list.push(score);
    }
}

fn skip_to_start(reader: &mut BufReader<File>, line: &mut String, prefix: &str) {
    loop {
        match reader.read_line(line) {
            Ok(0) => return,
            Ok(_) => {}
            Err(err) => return println!("Failed to read line: {err}"),
        }

        if line.starts_with(prefix) {
            return;
        }

        line.clear();
    }
}
