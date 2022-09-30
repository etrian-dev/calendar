use log::{error, warn};
use std::fs::{self, File};
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::result::Result;
use std::env;

use calendar_lib::calendar_error::CalendarError;
use calendar_lib::cli::{self, Cli, Commands};
use calendar_lib::calendar::Calendar;

fn read_calendar(p: &Path) -> Result<Calendar, CalendarError> {
    let p2 = &p.with_extension("json");
    if Path::exists(p2) {
        let f = File::open(p2)?;
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
    let dir_iter = fs::read_dir(p)?;

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
    let dir_iter = fs::read_dir(p)
        .expect(&format!("Calendar not found: {}", cal_file.display()));
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
                known_cals.push((read_calendar(&p.with_file_name(stem)), p.clone()));
            }
        }
    }
    println!("Known calendars: ");
    for cal in known_cals {
        if let (Ok(cal), path) = cal {
            println!(
                "{} (owned by {}) @ {}",
                cal.get_name(),
                if cal.get_owner().is_empty() {
                    "<unknown>"
                } else {
                    cal.get_owner()
                },
                path.display()
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

    let mut data_dir = std::env::current_dir()
        .expect("Cannot access the current directory");
    data_dir.push("data");
    if let Err(e) = fs::create_dir_all(data_dir.as_path()) {
        error!("Data directory creation failed: {e}");
        return;
    }

    let mut readonly = false;
    let res = match args {
        cli::Cli { view: Some(s), .. } => {
            readonly = true;
            read_calendar(&data_dir.join(Path::new(&s)))
        }
        cli::Cli { edit: Some(s), .. } => 
            read_calendar(&data_dir.join(Path::new(&s))),
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
            readonly = true;
            list_calendars(data_dir.as_path());
            // NOTE: this value is ignored
            Ok(Calendar::default())
        }
        cli::Cli { subcommand: Some(_), list: false, .. } => {
            // FIXME: maybe use the default calendar and allow only reads on it
           warn!("Unspecified calendar: aborting.");
           eprintln!("Unspecified calendar: aborting.");
           return;
		}
        _ => {
            let a: String = env::args().collect();
            warn!("Unrecognized command or option: {}", a);
            eprintln!("Unrecognized command or option: {}", a);
            return;
        }
    };
    let mut cal = res.expect("Error opening the calendar");
    let result = match (args.subcommand, readonly) {
        (Some(Commands::Add(x)), false) => match cli::handle_add(&mut cal, x) {
            Ok(x) => x,
            Err(e) => {
                error!("{}", e);
                false
            }
        },
        (Some(Commands::Remove(rm)), false) => cli::handle_remove(&mut cal, rm),
        (Some(Commands::List(l)), _) => cli::handle_list(&cal, l),
        (Some(Commands::Set(params)), false) => cli::handle_params(&mut cal, params),
        (Some(_), true) => {
            warn!(
                "Calendar {} cannot be modified! (rerun with --edit)",
                cal.get_name()
            );
            eprintln!(
				"Calendar {} cannot be modified! (rerun with --edit)",
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
        eprintln!("Cannot write calendar {} to {}", cal, data_dir.display());
    }
}
