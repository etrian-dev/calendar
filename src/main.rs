mod calendar;
mod calendar_error;
mod cli;
mod event;

use crate::calendar::Calendar;
use crate::cli::{Cli, Commands};

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

fn read_calendar(p: &Path) -> Calendar {
    if Path::exists(p) {
        let f = File::open(p).unwrap();
        let reader = BufReader::new(f);
        if let Ok(cal) = serde_json::from_reader(reader) {
            return cal;
        }
    }
    return Calendar::new("default");
}

fn write_calendar(cal: &Calendar, p: &Path) -> bool {
    let f = File::create(p).unwrap();
    let writer = BufWriter::new(f);
    match serde_json::to_writer_pretty(writer, cal) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    let args = Cli::parse_cli();

    let mut cal = read_calendar(Path::new("calendar.json"));

    match args.subcommand {
        Commands::Add(x) => cli::handle_add(&mut cal, x),
        Commands::Remove(rm) => cli::handle_remove(&mut cal, rm),
        Commands::List(l) => cli::handle_list(&cal, l),
    }

    write_calendar(&cal, Path::new("calendar.json"));
}
