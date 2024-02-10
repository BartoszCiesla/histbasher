use anyhow::Context;
use chrono::{Local, NaiveDateTime};
use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

const LINE_REGEX: &str = r"^(?<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \| (?<command>.*)";

#[derive(Parser)]
struct Cli {
    input: String,
    output: String,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    println!("Input file: {}", args.input);
    println!("Output file: {}", args.output);
    let mut input_lines = 0;
    let mut output_lines = 0;

    let input_file = File::open(Path::new(&args.input))
        .with_context(|| format!("Could not open input file {}", args.input))?;
    let output_file = File::create(Path::new(&args.output))
        .with_context(|| format!("Could not create output file {}", args.output))?;

    let reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);
    let regex =
        Regex::new(LINE_REGEX).with_context(|| format!("Could not create regex {LINE_REGEX}"))?;

    let local_timezone = Local::now().timezone();

    for line in reader.lines().map_while(Result::ok) {
        input_lines += 1;
        if let Some(captures) = regex.captures(&line) {
            let timestamp = captures
                .name("timestamp")
                .with_context(|| format!("Could not find timestamp in line {line}"))?
                .as_str();
            let date_time = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
                .with_context(|| format!("Could not parse timestamp {timestamp}"))?
                .and_local_timezone(local_timezone)
                .unwrap();
            let unix_timestamp = date_time.timestamp();
            let command = captures
                .name("command")
                .with_context(|| format!("Could not find command in line {line}"))?
                .as_str();
            write!(writer, "#{unix_timestamp}\n{command}\n",)
                .with_context(|| format!("Could not write output for line {line}"))?;
            output_lines += 1;
        }
    }

    println!("{input_lines} lines read");
    println!("{output_lines} lines written");
    let diff = input_lines - output_lines;
    println!("{diff} lines could not be parsed");

    Ok(())
}
