mod calendar;
mod calendar_error;
mod cli;
mod event;

use calendar_error::CalendarError;

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

    let res = match args {
        cli::Cli { view: Some(s), .. } => read_calendar(&data_dir.join(Path::new(&s))),
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
        _ => Ok(Calendar::new("default", "calendar")),
    };
    if let Err(e) = res {
        eprintln!("{:?}", e);
        return;
    }
    let mut cal = res.unwrap();

    let result = match args.subcommand {
        Some(Commands::Add(x)) => cli::handle_add(&mut cal, x),
        Some(Commands::Remove(rm)) => cli::handle_remove(&mut cal, rm),
        Some(Commands::List(l)) => cli::handle_list(&cal, l),
        None => true, // no commands to perform => ok to save result
    };

    if result {
        write_calendar(
            &cal,
            &data_dir.join(Path::new(cal.get_name()).with_extension("json")),
        );
    }
}
