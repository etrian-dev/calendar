use std::ffi::OsStr;
use std::io::Read;
use std::{fs, path};

use chrono::{Datelike, NaiveDateTime, Timelike};
use clap::{Args, Parser, Subcommand};
use icalendar::Property;

use crate::calendar::Calendar;
use crate::event::Event;

/// Simple calendar program
#[derive(Parser)]
#[clap(author,version,about,long_about=None)]
pub struct Cli {
    /// Specifies a subcommand
    #[clap(subcommand)]
    pub subcommand: Commands,
    /// Create a calendar
    #[clap(short, long)]
    create: Option<String>,
    /// Delete a calendar
    #[clap(short, long)]
    delete: Option<String>,
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
pub struct Add {
    /// The event's title
    title: Option<String>,
    /// The event's description
    description: Option<String>,
    /// The event's start date. Supported formats: %d/%m/%yyyy
    start_date: Option<String>,
    /// The event's start time. Supported formats: %H:%M
    start_time: Option<String>,
    /// The event's duration, expressed in hours (floating point)
    duration: Option<String>,
    #[clap(long)]
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

pub fn handle_ics(fpath: &str) -> std::result::Result<Vec<Event>, String> {
    let path = path::Path::new(fpath);
    if path.exists() && path.extension().unwrap_or(&OsStr::new("ics")) == "ics" {
        let mut ics_file = fs::File::open(path).expect("Cannot open file");
        let mut buf = String::new();
        if let Err(e) = ics_file.read_to_string(&mut buf) {
            return Err(String::from(format!("Cannot read ics file: {}", e)));
        } else {
            let str_unfolded = icalendar::parser::unfold(&buf);
            let cal = icalendar::parser::read_calendar(&str_unfolded)?;
            let mut events = Vec::new();
            for comp in cal.components {
                let mut e = Event::default();
                if comp.name == "VEVENT" {
                    for prop in comp.properties.iter() {
                        if prop.name.as_str() == "SUMMARY" {
                            e.set_title(prop.val.as_str());
                        }
                        if prop.name.as_str() == "DESCRIPTION" {
                            e.set_description(prop.val.as_str());
                        }
                        if prop.name.as_str() == "DTSTART" {
                            let (date, time) = ics_parse_date_time(&prop);
                            e.set_start_date((date.day(), date.month(), date.year()));
                            e.set_start_time((time.hour(), time.minute(), time.second()));
                        }
                        if prop.name.as_str() == "DTEND" {
                            let (end_date, end_time) = ics_parse_date_time(&prop);
                            let start_date = e.get_start_date();
                            let start_time = e.get_start_time();
                            let dur = end_date.and_time(end_time) - start_date.and_time(start_time);
                            e.set_duration(&dur);
                        }
                    }
                }
                events.push(e);
            }
            return Ok(events);
        }
    }
    Err(String::from("path does not exist"))
}

pub fn handle_add(cal: &mut Calendar, x: Add) -> () {
    // if the flag --from-file is given it takes precedence
    if let Some(path) = x.from_file {
        match handle_ics(&path) {
            Ok(events) => {
                for ev in events {
                    println!("{}", ev);
                    cal.add_event(ev);
                }
            }
            Err(e) => eprintln!("Failed parsing .ics file: {}", e),
        }
        return;
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
    let ev = Event::new(&title, &description, &start_date, &start_time, duration);
    cal.add_event(ev)
}

pub fn handle_list(cal: &Calendar, x: Filter) -> () {
    let events = match x {
        Filter { today: true, .. } => cal.list_events_today(),
        Filter { week: true, .. } => cal.list_events_week(),
        Filter { month: true, .. } => cal.list_events_month(),
        _ => cal.list_events(),
    };
    println!("{}", cal);
    for ev in events {
        println!("{}", ev);
    }
}

pub fn handle_remove(cal: &mut Calendar, x: Remove) -> () {
    match cal.remove_event(x.eid) {
        Ok(ev) => println!("Event {:?} removed successfully", ev),
        Err(e) => println!("Error: {}", e),
    }
}
