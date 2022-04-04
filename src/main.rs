mod calendar;
mod calendar_error;
mod event;

use calendar::Calendar;
use clap::{Args, Parser, Subcommand};
use event::Event;

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

/// Simple calendar program
#[derive(Parser)]
#[clap(author,version,about,long_about=None)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Commands,
    /// Create a calendar
    #[clap(short, long)]
    create: Option<String>,
    /// Delete a calendar
    #[clap(short, long)]
    delete: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a new event
    Add(Add),
    /// Removes an event, given its eid
    Remove(Remove),
    /// Lists events with some filter
    List(Filter),
}

#[derive(Args)]
struct Add {
    title: String,
    description: String,
    start_date: String,
    start_time: String,
    duration: String,
}

#[derive(Args)]
struct Remove {
    eid: u64,
}

#[derive(Args)]
struct Filter {
    /// filters events occurring today
    today: Option<bool>,
    /// filters events starting from the given date
    from: Option<String>,
    /// filters events until the given date
    to: Option<String>,
}

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

fn handle_add(cal: &mut Calendar, x: &Add) -> () {
    let ev = Event::new(
        &x.title,
        &x.description,
        &(x.start_date.clone() + &x.start_time),
        x.duration.parse::<f32>().unwrap_or(1.0),
    );
    cal.add_event(ev)
}

fn handle_list(cal: &Calendar, x: &Filter) -> () {
    let events = match x {
        Filter {
            today: Some(true), ..
        } => cal.list_events_today(),
        _ => cal.list_events(|x| {
            let mut v = Vec::new();
            for elem in x {
                v.push(elem);
            }
            v
        }),
    };
    println!("{:?}", events);
}

fn handle_remove(cal: &mut Calendar, x: &Remove) -> () {
    match cal.remove_event(x.eid) {
        Ok(ev) => println!("Event {:?} removed successfully", ev),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    let args = Cli::parse();

    let mut cal = read_calendar(Path::new("calendar.json"));

    match &args.subcommand {
        Commands::Add(x) => handle_add(&mut cal, x),
        Commands::Remove(rm) => handle_remove(&mut cal, rm),
        Commands::List(l) => handle_list(&cal, l),
    }

    write_calendar(&cal, Path::new("calendar.json"));
}
