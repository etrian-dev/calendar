use clap::{Args, Parser, Subcommand};

use crate::calendar::Calendar;
use crate::event::Event;

/// Simple calendar program
#[derive(Parser)]
#[clap(author,version,about,long_about=None)]
pub struct Cli {
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
    title: String,
    description: String,
    start_date: String,
    start_time: String,
    duration: String,
}

#[derive(Args)]
pub struct Remove {
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

pub fn handle_add(cal: &mut Calendar, x: &Add) -> () {
    let ev = Event::new(
        &x.title,
        &x.description,
        &x.start_date,
        &x.start_time,
        x.duration.parse::<f32>().unwrap_or(1.0),
    );
    cal.add_event(ev)
}

pub fn handle_list(cal: &Calendar, x: &Filter) -> () {
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

pub fn handle_remove(cal: &mut Calendar, x: &Remove) -> () {
    match cal.remove_event(x.eid) {
        Ok(ev) => println!("Event {:?} removed successfully", ev),
        Err(e) => println!("Error: {}", e),
    }
}
