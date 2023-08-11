use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::path::Path;
use std::result::Result;

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use clap::{ArgGroup, Args, Parser, Subcommand};
use icalendar::parser::{Component, Property};

use crate::calendar::Calendar;
use crate::calendar_error::CalendarError;
use crate::event::Event;

use log::{error, info, warn};

/// Simple calendar program
#[derive(Parser)]
#[clap(author,version,about,long_about=None)]
pub struct Cli {
    /// Specifies a subcommand
    #[clap(subcommand)]
    pub subcommand: Option<Commands>,
    /// View this calendar (if it exists)
    #[clap(short, long)]
    pub view: Option<String>,
    /// Edit an existing calendar
    #[clap(short, long)]
    pub edit: Option<String>,
    /// Create a calendar
    #[clap(short, long)]
    pub create: Option<String>,
    /// Specify the calendar's name
    #[clap(short, long)]
    pub name: Option<String>,
    /// Delete a calendar
    #[clap(short, long)]
    pub delete: Option<String>,
    /// List all known calendars
    #[clap(short, long)]
    pub list: bool,
}

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

fn create_calendar(calname: &str, cal_owner: &str, p: &Path) -> Result<Calendar, CalendarError> {
    let cal_file = p.join(calname).with_extension("json");
    let dir_iter = fs::read_dir(p)?;

    for entry in dir_iter.flatten() {
        if entry.path() == cal_file {
            return Err(CalendarError::CalendarAlreadyExists(calname.to_string()));
        }
    }
    Ok(Calendar::new(calname, cal_owner))
}

fn delete_calendar(calname: &str, p: &Path) -> bool {
    let cal_file = p.join(calname).with_extension("json");
    let dir_iter = fs::read_dir(p).expect(&format!("Calendar not found: {}", cal_file.display()));
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

pub fn save_calendar(cal: &Calendar, p: &Path) -> bool {
    let f = File::create(p).unwrap();
    let writer = BufWriter::new(f);
    serde_json::to_writer_pretty(writer, cal).is_ok()
}

impl Cli {
    pub fn parse_cli() -> Cli {
        Cli::parse()
    }

    pub fn exec_commands(
        args: &Cli,
        data_dir: &Path,
    ) -> (bool, Result<Option<Calendar>, CalendarError>) {
        let mut readonly = false;
        let res = match args {
            Cli { view: Some(s), .. } | Cli { edit: Some(s), .. } => {
                if args.edit.is_none() {
                    readonly = true;
                }
                read_calendar(&data_dir.join(Path::new(&s))).and_then(|c| Ok(Some(c)))
            }
            Cli {
                create: Some(owner),
                name,
                ..
            } => {
                let mut calname = owner;
                if let Some(n) = name {
                    calname = n;
                }
                create_calendar(calname, owner, data_dir).and_then(|c| Ok(Some(c)))
            }
            Cli {
                delete: Some(s), ..
            } => {
                if delete_calendar(s, data_dir) {
                    Ok(None)
                } else {
                    Err(CalendarError::CalendarNotFound(s.to_string()))
                }
            }
            Cli { list: true, .. } => {
                readonly = true;
                list_calendars(data_dir);
                // NOTE: this value is ignored
                Ok(None)
            }
            Cli {
                subcommand: Some(_),
                list: false,
                ..
            } => {
                // FIXME: maybe use the default calendar and allow only reads on it
                warn!("Unspecified calendar: aborting.");
                //eprintln!("Unspecified calendar: aborting.");
                Err(CalendarError::CalendarNotFound(
                    "Unspecified calendar: aborting.".to_string(),
                ))
            }
            _ => {
                let a: String = env::args().collect();
                warn!("Unrecognized command or option: {}", a);
                //eprintln!("Unrecognized command or option: {}", a);
                Err(CalendarError::Unknown(format!(
                    "Unrecognized command or option: {a}"
                )))
            }
        };
        (readonly, res)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Adds a new event
    Add(Add),
    /// Removes an event, given its eid
    Remove(Remove),
    /// Edit an event, given its eid
    Edit(Edit),
    /// Lists events with some filter
    List(Filter),
    /// Sets some parameter about the calendar
    Set(CalParams),
}

#[derive(Args)]
#[clap(group(ArgGroup::new("input").multiple(true)))]
pub struct Add {
    #[clap(group = "input")]
    /// The event's title
    title: Option<String>,
    #[clap(group = "input")]
    /// The event's description
    description: Option<String>,
    #[clap(group = "input")]
    /// The event's start date. Supported formats: %d/%m/%yyyy
    start_date: Option<String>,
    #[clap(group = "input")]
    /// The event's start time. Supported formats: %H:%M
    start_time: Option<String>,
    #[clap(group = "input")]
    /// The event's duration, expressed in hours (floating point)
    duration: Option<String>,
    #[clap(group = "input")]
    /// The event's location, as a string
    location: Option<String>,
    #[clap(group = "input")]
    /// The event's recurrence
    recurrence: Option<String>,
    #[clap(group = "input")]
    // The event's tags
    tags: Vec<String>,
    #[clap(long, group = "ics", conflicts_with = "input")]
    /// Load the event to be added from an .ics file (iCalendar format)
    from_file: Option<String>,
}

#[derive(Args)]
#[clap(group(ArgGroup::new("input").multiple(true)))]
pub struct Edit {
    #[clap(group = "input")]
    /// The event eid to be modified
    eid: u64,
    #[clap(group = "input")]
    /// The event's title
    title: Option<String>,
    #[clap(group = "input")]
    /// The event's description
    description: Option<String>,
    #[clap(group = "input")]
    /// The event's start date. Supported formats: %d/%m/%yyyy
    start_date: Option<String>,
    #[clap(group = "input")]
    /// The event's start time. Supported formats: %H:%M
    start_time: Option<String>,
    #[clap(group = "input")]
    /// The event's duration, expressed in hours (floating point)
    duration: Option<String>,
    #[clap(group = "input")]
    /// The event's location, as a string
    location: Option<String>,
    #[clap(group = "input")]
    /// The event's recurrence
    recurrence: Option<String>,
    #[clap(group = "input")]
    // The event's tags
    tags: Vec<String>,
    #[clap(long, group = "ics", conflicts_with = "input")]
    /// Load the events to be modified from an .ics file (iCalendar format)
    from_file: Option<String>,
}

#[derive(Args)]
pub struct Remove {
    /// The id of the event to be removed
    eid: u64,
    #[clap(short, long)]
    /// Delete all events starting at the given date
    from: Option<String>,
    #[clap(short, long)]
    /// Delete all events until the given date
    to: Option<String>,
    #[clap(short, long)]
    /// Filter function for events to be removed
    filter: Option<String>,
    #[clap(short, long)]
    /// Removes all events in the calendar
    all: bool,
}

#[derive(Args)]
pub struct Filter {
    /// filters events occurring today
    #[clap(short, long)]
    today: bool,
    /// filters events occurring this week
    #[clap(short, long)]
    week: bool,
    /// filters events occurring this month
    #[clap(short, long)]
    month: bool,
    /// filters events starting from the given date
    #[clap(long)]
    from: Option<String>,
    /// filters events until the given date
    #[clap(long)]
    until: Option<String>,
    /// filters by tag
    #[clap(long)]
    tag: Option<String>,
}

#[derive(Args)]
pub struct CalParams {
    #[clap(long)]
    /// Sets the calendar's name
    name: Option<String>,
    #[clap(long)]
    /// Sets the calendar's owner
    owner: Option<String>,
}

fn ics_parse_date_time(prop: &Property) -> (chrono::NaiveDate, chrono::NaiveTime) {
    let dt = NaiveDateTime::parse_from_str(prop.val.as_str(), "%Y%m%dT%H%M%SZ")
        .expect("Failed to parse the DTSTART field");
    (dt.date(), dt.time())
}

fn match_property(ev: &mut Event, comp: Component) {
    for prop in comp.properties.iter() {
        match prop.name.as_str() {
            "SUMMARY" => ev.set_title(prop.val.as_str()),
            "DESCRIPTION" => ev.set_description(prop.val.as_str()),
            "DTSTART" => {
                let (date, time) = ics_parse_date_time(prop);
                ev.set_start_date((date.day(), date.month(), date.year()));
                ev.set_start_time((time.hour(), time.minute(), time.second()));
            }
            "DTEND" => {
                let (end_date, end_time) = ics_parse_date_time(prop);
                let start_date = ev.get_start_date();
                let start_time = ev.get_start_time();
                let dur = end_date.and_time(end_time) - start_date.and_time(start_time);
                ev.set_duration(&dur);
            }
            "LOCATION" => ev.set_location(prop.val.as_str()),
            "RRULE" => {
                let mut rec = String::new();
                for param in prop.val.as_str().split(';') {
                    let x: Vec<&str> = param.splitn(2, '=').collect();
                    match x[0] {
                        // See https://icalendar.org/iCalendar-RFC-5545/3-3-10-recurrence-rule.html
                        "FREQ" => rec = x[1].to_owned() + " " + &rec,
                        "COUNT" => rec.push_str(&(x[1].to_owned() + " ")),
                        "INTERVAL" => rec.push_str(&(x[1].to_owned() + " ")),
                        _ => (),
                    }
                }
                ev.set_recurrence(&rec)
            }
            // property ignored by the event struct
            _ => (),
        }
    }
}

fn handle_ics(fpath: &str) -> Result<Vec<Event>, String> {
    let path = Path::new(fpath);
    if path.exists() && path.extension().unwrap_or(OsStr::new("ics")) == "ics" {
        let ics_file = fs::File::open(path);
        if let Err(e) = ics_file {
            return Err(e.to_string());
        }
        let mut buf = String::new();
        if let Err(e) = ics_file.unwrap().read_to_string(&mut buf) {
            return Err(format!("Cannot read ics file: {}", e));
        } else {
            // File read into the buf String: parse it with the iCalendar library
            let str_unfolded = icalendar::parser::unfold(&buf);
            return match icalendar::parser::read_calendar(&str_unfolded) {
                Ok(cal) => {
                    let mut events = Vec::new();
                    for comp in cal.components {
                        if comp.name == "VEVENT" {
                            let mut e = Event::default();
                            match_property(&mut e, comp);
                            events.push(e);
                        }
                    }
                    Ok(events)
                }
                Err(s) => Err(format!("Error parsing {}: {}", path.display(), s)),
            };
        }
    }
    Err(format!(
        "{} does not exists or is not a valid .ics file",
        path.display()
    ))
}

pub fn handle_add(cal: &mut Calendar, x: Add) -> Result<bool, CalendarError> {
    // if the flag --from-file is given it takes precedence
    if let Some(path) = x.from_file {
        match handle_ics(&path) {
            Ok(events) => {
                let mut imported: usize = 0;
                let total_events = events.len();
                for ev in events {
                    if cal.add_event(ev) {
                        imported += 1;
                    }
                }
                info!(
                    "Imported {} (total: {}) events from {}",
                    imported, total_events, &path
                );
                println!(
                    "Imported {} (total: {}) events from {}",
                    imported, total_events, &path
                );
                Ok(true)
            }
            Err(e) => Err(CalendarError::IcsParsingFailed(e)),
        }
    } else {
        let default_values = Event::default();
        let title = match x.title {
            Some(val) => val,
            None => default_values.get_title().to_string(),
        };
        let description = match x.description {
            Some(val) => val,
            None => default_values.get_description().to_string(),
        };
        let start_date = match x.start_date {
            Some(val) => val,
            None => default_values.get_start_date().to_string(),
        };
        let start_time = match x.start_time {
            Some(val) => val,
            None => default_values.get_start_time().to_string(),
        };
        let duration = match x.duration {
            Some(val) => val.parse().unwrap(),
            None => default_values.get_duration() as f32,
        };
        let loc = x.location.as_deref();
        let rec = x.recurrence.as_deref();

        let tags = if !x.tags.is_empty() {
            Some(x.tags)
        } else {
            None
        };

        let ev = Event::new(
            &title,
            &description,
            &start_date,
            &start_time,
            duration,
            loc,
            rec,
            tags,
        );
        Ok(cal.add_event(ev))
    }
}

pub fn handle_edit(cal: &mut Calendar, x: Edit) -> Result<bool, CalendarError> {
    if let Some(_) = x.from_file {
        return Err(CalendarError::Unknown("Unimplemented!".to_owned()));
    }
    match cal.get_event(x.eid) {
        Ok(ev) => {
            if let Some(title) = x.title {
                ev.set_title(&title);
            }
            if let Some(descr) = x.description {
                ev.set_description(&descr);
            }
            if let Some(s) = x.start_date {
                let date_formats = vec!["%d/%m/%Y", "%Y-%m-%d"];
                for fmt in date_formats {
                    if let Ok(val) = NaiveDate::parse_from_str(&s, fmt) {
                        ev.set_start_date((val.day(), val.month(), val.year()));
                        break;
                    }
                }
            }
            if let Some(s) = x.start_time {
                let time_formats = vec!["%H:%M", "%H:%M:%S"];
                for fmt in time_formats {
                    if let Ok(val) = NaiveTime::parse_from_str(&s, fmt) {
                        ev.set_start_time((val.hour(), val.minute(), val.second()));
                        break;
                    }
                }
            }
            if let Some(duration) = x.duration {
                ev.set_duration(&Duration::hours(duration.parse::<i32>().unwrap().into()));
            }
            if let Some(loc) = x.location {
                ev.set_location(&loc);
            }
            if let Some(rec) = x.recurrence {
                ev.set_recurrence(&rec);
            }
            if x.tags.len() > 0 {
                ev.set_tags(x.tags);
            }
            Ok(true)
        }
        _ => Err(CalendarError::Unknown("Unimplemented!".to_string())),
    }
}

pub fn handle_list(cal: &Calendar, x: Filter) -> bool {
    let dt = Local::now().naive_local();
    /// TODO: error handling in the match arms abstracted into a function
    let events = match x {
        Filter { today: true, .. } => {
            let start = dt.with_hour(0).unwrap().with_minute(0).unwrap();
            let end = dt.with_hour(23).unwrap().with_minute(59).unwrap();
            cal.list_events_between(Some(start), Some(end))
        }
        Filter { week: true, .. } => {
            let weekday = dt.weekday();
            let start = dt
                .with_day(dt.day() - weekday.num_days_from_monday())
                .unwrap()
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap();
            let end = dt
                .with_day(dt.day() - weekday.num_days_from_sunday())
                .unwrap()
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap();
            cal.list_events_between(Some(start), Some(end))
        }
        Filter { month: true, .. } => {
            let start = dt
                .with_day(1)
                .unwrap()
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap();
            let end = dt
                .with_day(31)
                .unwrap_or(dt.with_day(30).unwrap())
                .with_hour(23)
                .unwrap()
                .with_minute(59)
                .unwrap();
            cal.list_events_between(Some(start), Some(end))
        }
        Filter { tag: Some(tag), .. } => cal.list_events_tagged(tag),
        Filter {
            today: false,
            week: false,
            month: false,
            from: None,
            until: None,
            tag: None,
        } => {
            // by default list all events starting from today
            let start = dt.with_hour(0).unwrap().with_minute(0).unwrap();
            cal.list_events_between(Some(start), None)
        }
        Filter {
            from: x, until: y, ..
        } => {
            /// FIXME: Some error handling here
            let from_dt = match x {
                Some(s) => Some(
                    NaiveDateTime::parse_from_str(&s, "%d/%m/%Y")
                        .unwrap_or(chrono::NaiveDateTime::MIN),
                ),
                None => None,
            };
            let until_dt = match y {
                Some(s) => Some(
                    NaiveDateTime::parse_from_str(&s, "%d/%m/%Y")
                        .unwrap_or(chrono::NaiveDateTime::MAX),
                ),
                None => None,
            };
            cal.list_events_between(from_dt, until_dt)
        }
    };
    println!("{}", cal);
    for ev in events {
        println!("{}", ev);
    }
    true
}

pub fn handle_remove(cal: &mut Calendar, x: Remove) -> bool {
    if x.all {}
    match x {
        Remove { all: true, .. } => {
            let calsize = cal.get_size();
            cal.clear();
            println!(
                "Calendar {} cleared ({} events removed)",
                cal.get_name(),
                calsize
            );
            true
        }
        Remove {
            eid,
            from: None,
            to: None,
            filter: None,
            all: false,
        } => match cal.remove_event(eid) {
            Ok(ev) => {
                println!("Event \n{ev}\nremoved successfully");
                true
            }
            Err(e) => {
                error!("Failed to remove event {}: {e}", eid);
                false
            }
        },
        // TODO: implement other filters
        _ => {
            error!("Unknown remotion filter");
            false
        }
    }
}

pub fn handle_params(cal: &mut Calendar, params: CalParams) -> bool {
    if let Some(s) = params.name {
        cal.set_name(&s);
    }
    if let Some(s) = params.owner {
        cal.set_owner(&s);
    }
    true
}
