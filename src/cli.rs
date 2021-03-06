use std::ffi::OsStr;
use std::io::Read;
use std::result::Result;
use std::{fs, path};

use chrono::{Datelike, NaiveDateTime, Timelike};
use clap::{ArgGroup, Args, Parser, Subcommand};

use crate::calendar::Calendar;
use crate::event::Event;

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
    /// Create a calendar
    #[clap(short, long)]
    pub create: Option<String>,
    /// Delete a calendar
    #[clap(short, long)]
    pub delete: Option<String>,
}

impl Cli {
    pub fn parse_cli() -> Cli {
        Cli::parse()
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Adds a new event
    Add(Add),
    /// Removes an event, given its eid
    Remove(Remove),
    /// Lists events with some filter
    List(Filter),
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
    #[clap(long, group = "ics", conflicts_with = "input")]
    /// Load the event to be added from an .ics file (iCalendar format)
    from_file: Option<String>,
}

#[derive(Args)]
pub struct Remove {
    /// The id of the event to be removed
    eid: u64,
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
}

fn ics_parse_date_time(
    prop: &icalendar::parser::Property,
) -> (chrono::NaiveDate, chrono::NaiveTime) {
    let dt = NaiveDateTime::parse_from_str(prop.val.as_str(), "%Y%m%dT%H%M%SZ")
        .expect("Failed to parse the DTSTART field");
    (dt.date(), dt.time())
}

fn match_property(ev: &mut Event, comp: icalendar::parser::Component) {
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
                        "FREQ" => rec = x[1].to_owned() + " " + &rec,
                        "COUNT" => rec.push_str(x[1]),
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
    let path = path::Path::new(fpath);
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
            let cal = icalendar::parser::read_calendar(&str_unfolded)?;
            let mut events = Vec::new();
            for comp in cal.components {
                if comp.name == "VEVENT" {
                    let mut e = Event::default();
                    match_property(&mut e, comp);
                    events.push(e);
                }
            }
            return Ok(events);
        }
    }
    Err(format!(
        "{} does not exists or is not an .ics file",
        path.display()
    ))
}

pub fn handle_add(cal: &mut Calendar, x: Add) -> bool {
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
                println!(
                    "Imported {} (total: {}) events from {}",
                    imported, total_events, &path
                );
            }
            Err(e) => println!("Failed parsing .ics file: {}", e),
        }
        // whether events were succeddfully parsed or not, return true to write changes
        return true;
    }

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

    let ev = Event::new(
        &title,
        &description,
        &start_date,
        &start_time,
        duration,
        loc,
        rec,
    );
    cal.add_event(ev)
}

pub fn handle_list(cal: &Calendar, x: Filter) -> bool {
    let events = match x {
        Filter { today: true, .. } => cal.list_events_today(),
        Filter { week: true, .. } => cal.list_events_week(),
        Filter { month: true, .. } => cal.list_events_month(),
        Filter {
            today: false,
            week: false,
            month: false,
            from: None,
            until: None,
        } => {
            let today = format!("{}", chrono::Local::today().format("%d/%m/%Y"));
            cal.list_events_between(Some(today), None)
        }
        Filter {
            from: x, until: y, ..
        } => cal.list_events_between(x, y),
    };
    println!("{}", cal);
    for ev in events {
        println!("{}", ev);
    }
    true
}

pub fn handle_remove(cal: &mut Calendar, x: Remove) -> bool {
    match cal.remove_event(x.eid) {
        Ok(ev) => {
            println!("Event \n{ev}\nremoved successfully");
            true
        }
        Err(e) => {
            println!("Failed to remove event {}: {e}", x.eid);
            false
        }
    }
}
