mod calendar;
mod calendar_error;
mod cli;
mod event;

use calendar_error::CalendarError;
use log::warn;

use crate::calendar::Calendar;
use crate::cli::{Cli, Commands};

use std::fs::{self, File};
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::result::Result;

fn read_calendar(p: &Path) -> Result<Calendar, CalendarError> {
    let p2 = &p.with_extension("json");
    if Path::exists(p2) {
        let f = File::open(p2).unwrap();
        let reader = BufReader::new(f);
        if let Ok(cal) = serde_json::from_reader(reader) {
            return Ok(cal);
        }
    }
    Err(CalendarError::CalendarNotFound(
        p2.to_string_lossy().to_string(),
    ))
}

fn write_calendar(cal: &Calendar, p: &Path) -> bool {
    let f = File::create(p).unwrap();
    let writer = BufWriter::new(f);
    serde_json::to_writer_pretty(writer, cal).is_ok()
}

fn create_cal(calname: &str, p: &Path) -> Result<Calendar, CalendarError> {
    let cal_file = p.join(calname).with_extension("json");
    let dir_iter = fs::read_dir(p).unwrap();

    for entry in dir_iter.flatten() {
        if entry.path() == cal_file {
            return Err(CalendarError::CalendarAlreadyExists(calname.to_string()));
        }
    }
    // FIXME: currently setting the owner is unsupported
    Ok(Calendar::new("", calname))
}

fn delete_cal(calname: &str, p: &Path) -> bool {
    let cal_file = p.join(calname).with_extension("json");
    let dir_iter = fs::read_dir(p).unwrap();
    for entry in dir_iter.flatten() {
        if entry.path() == cal_file {
            return fs::remove_file(entry.path()).is_ok();
        }
    }
    false
}

fn list_calendars(p: &Path) {
    let mut known_cals = Vec::new();
    let dir_iter = fs::read_dir(p).unwrap();
    for ent in dir_iter.flatten() {
        let p = ent.path();
        let ext = p.extension();
        let stem = p.file_stem().unwrap();
        if let Some(s) = ext {
            if s.eq("json") {
                known_cals.push(read_calendar(&p.with_file_name(stem)));
            }
        }
    }
    println!("Known calendars: ");
    for cal in known_cals {
        if let Ok(c) = cal {
            println!(
                "{} (owned by {})",
                c.get_name(),
                if c.get_owner().is_empty() {
                    "<unknown>"
                } else {
                    c.get_owner()
                }
            );
        } else {
            eprintln!("Error for calendar!");
        }
    }
}

fn main() {
    // Initialize logging
    env_logger::init();

    let args = Cli::parse_cli();

    let mut data_dir = std::env::current_dir().unwrap();
    data_dir.push("data");
    if let Err(e) = fs::create_dir_all(data_dir.as_path()) {
        println!("Data directory creation failed: {e}");
        return;
    }

    let mut readonly = false;
    let res = match args {
        cli::Cli { view: Some(s), .. } => {
            readonly = true;
            read_calendar(&data_dir.join(Path::new(&s)))
        }
        cli::Cli { edit: Some(s), .. } => read_calendar(&data_dir.join(Path::new(&s))),
        cli::Cli {
            create: Some(s), ..
        } => create_cal(&s, data_dir.as_path()),
        cli::Cli {
            delete: Some(s), ..
        } => {
            if delete_cal(&s, data_dir.as_path()) {
                return;
            } else {
                Err(CalendarError::CalendarNotFound(s))
            }
        }
        cli::Cli { list: true, .. } => {
            list_calendars(data_dir.as_path());
            readonly = true;
            Ok(Calendar::default())
        }
        _ => Ok(Calendar::default()),
    };
    if let Err(e) = res {
        eprintln!("{:?}", e);
        return;
    }
    let mut cal = res.unwrap();

    let result = match (args.subcommand, readonly) {
        (Some(Commands::Add(x)), false) => match cli::handle_add(&mut cal, x) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        },
        (Some(Commands::Remove(rm)), false) => cli::handle_remove(&mut cal, rm),
        (Some(Commands::List(l)), _) => cli::handle_list(&cal, l),
        (Some(Commands::Set(params)), false) => cli::handle_params(&mut cal, params),
        (Some(_), true) => {
            eprintln!(
                "Calendar {} cannot be modified! (rerun without --view)",
                cal.get_name()
            );
            false
        }
        (None, _) => true, // no commands to perform => ok to save result
    };

    if result
        && !write_calendar(
            &cal,
            &data_dir.join(Path::new(cal.get_name()).with_extension("json")),
        )
    {
        warn!("Cannot write calendar {} to {}", cal, data_dir.display());
    }
}
