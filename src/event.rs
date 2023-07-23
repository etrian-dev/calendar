use chrono::{DateTime, Duration, Local, Months, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Result as fmtResult;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::result::Result;
use std::str::FromStr;
use std::vec;

use log::warn;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Cadence {
    Secondly,
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl FromStr for Cadence {
    type Err = ParseRecurrenceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "secondly" => Ok(Cadence::Secondly),
            "minutely" => Ok(Cadence::Minutely),
            "hourly" => Ok(Cadence::Hourly),
            "daily" => Ok(Cadence::Daily),
            "weekly" => Ok(Cadence::Weekly),
            "monthly" => Ok(Cadence::Monthly),
            "yearly" => Ok(Cadence::Yearly),
            _ => Err(ParseRecurrenceError::UnknownCadence(s.to_string())),
        }
    }
}

pub enum ParseRecurrenceError {
    UnknownCadence(String),
    BadFormat(String),
}
impl Display for ParseRecurrenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmtResult {
        match self {
            Self::UnknownCadence(s) => write!(f, "{} cannot be parsed as a Cadence", s),
            Self::BadFormat(s) => write!(f, "Failed to parse recurrence {}", s),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Recurrence {
    cadence: Cadence,
    repetitions: usize,
    interval: Option<usize>,
}

impl Recurrence {
    pub fn cadence(&self) -> &Cadence {
        &self.cadence
    }

    pub fn repetitions(&self) -> usize {
        self.repetitions
    }

    pub fn interval(&self) -> Option<usize> {
        self.interval
    }

    pub fn set_cadence(&mut self, new_cad: Cadence) {
        self.cadence = new_cad;
    }

    pub fn set_repetitions(&mut self, new_repeat: usize) {
        self.repetitions = new_repeat;
    }

    pub fn set_interval(&mut self, new_interval: Option<usize>) {
        self.interval = new_interval;
    }
}

impl Default for Recurrence {
    fn default() -> Self {
        Recurrence {
            cadence: Cadence::Weekly,
            repetitions: 0,
            interval: None,
        }
    }
}

fn duration_to_min<S>(dur: &Duration, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_i64(dur.num_minutes())
}

fn min_to_duration<'de, D>(de: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let x = i64::deserialize(de);
    match x {
        Ok(val) => Ok(Duration::minutes(val)),
        Err(e) => Err(e),
    }
}

fn parse_recurrence(s: &str) -> Option<Recurrence> {
    let components: Vec<&str> = s.split_ascii_whitespace().collect();
    if components.len() < 2 || components.len() > 3 {
        return None;
    }
    // Parse optional interval parameter
    let mut interv = None;
    if components.len() == 3 {
        if let Ok(val) = components[2].parse::<usize>() {
            interv = Some(val);
        }
    }
    let cad = Cadence::from_str(components[0]);
    let repeat = components[1].parse::<usize>();
    match (cad, repeat) {
        (Ok(c), Ok(val)) => {
            if val == 0 {
                return None;
            }
            return Some(Recurrence {
                cadence: c,
                repetitions: val,
                interval: interv,
            });
        }
        (_, _) => {
            return None;
        }
    }
}

pub fn next_occurrence(ev: &Event, cadence: &Cadence) -> (NaiveDateTime, NaiveDateTime) {
    let ev_start = ev.get_start_date().and_time(ev.get_start_time());
    let ev_end = ev_start + Duration::seconds(ev.get_duration());
    match cadence {
        Cadence::Secondly => (
            ev_start + Duration::seconds(1),
            ev_end + Duration::seconds(1),
        ),
        Cadence::Minutely => (
            ev_start + Duration::minutes(1),
            ev_end + Duration::minutes(1),
        ),
        Cadence::Hourly => (ev_start + Duration::hours(1), ev_end + Duration::hours(1)),
        Cadence::Daily => (ev_start + Duration::days(1), ev_end + Duration::days(1)),
        Cadence::Weekly => (ev_start + Duration::weeks(1), ev_end + Duration::weeks(1)),
        Cadence::Monthly => (
            NaiveDateTime::new(ev_start.date() + Months::new(1), ev_start.time()),
            NaiveDateTime::new(ev_end.date() + Months::new(1), ev_end.time()),
        ),
        Cadence::Yearly => (
            NaiveDateTime::new(ev_start.date() + Months::new(12), ev_start.time()),
            NaiveDateTime::new(ev_end.date() + Months::new(12), ev_end.time()),
        ),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct EventMetadata {
    tags: Vec<String>,
    creation: DateTime<Local>,
    modification: DateTime<Local>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        EventMetadata {
            tags: Vec::default(),
            creation: Local::now(),
            modification: Local::now(),
        }
    }
}

impl EventMetadata {
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags
    }
    pub fn get_tags(&self) -> Vec<String> {
        self.tags.clone()
    }
    pub fn get_creation(&self) -> DateTime<Local> {
        self.creation
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Event {
    title: String,
    description: String,
    start_date: NaiveDate,
    start_time: NaiveTime,
    #[serde(serialize_with = "duration_to_min")]
    #[serde(deserialize_with = "min_to_duration")]
    duration: Duration,
    location: String,
    recurrence: Option<Recurrence>,
    metadata: EventMetadata,
}

impl Event {
    pub fn new(
        event_title: &str,
        descr: &str,
        start_date: &str,
        start_time: &str,
        dur: f32,
        location: Option<&str>,
        recurr: Option<&str>,
        tags: Option<Vec<String>>,
    ) -> Event {
        let date_formats = vec!["%d/%m/%Y", "%Y-%m-%d"];
        let mut date = Err(());
        for fmt in date_formats {
            if let Ok(val) = NaiveDate::parse_from_str(start_date, fmt) {
                date = Ok(val);
                break;
            }
        }

        let time_formats = vec!["%H:%M", "%H:%M:%S"];
        let mut time = Err(());
        for fmt in time_formats {
            if let Ok(val) = NaiveTime::parse_from_str(start_time, fmt) {
                time = Ok(val);
                break;
            }
        }

        let d = Duration::hours((dur as i32).into());
        Event {
            // add a unique, random, event id
            title: event_title.to_string(),
            description: descr.to_string(),
            start_date: match date {
                Ok(date) => date,
                Err(_) => {
                    warn!(
                        "Unrecognized date format {}: defaults to current date",
                        start_date
                    );
                    Local::now().date_naive()
                }
            },
            start_time: match time {
                Ok(tm) => tm,
                Err(_) => {
                    warn!(
                        "Unrecognized time format {}: defaults to current time",
                        start_time
                    );
                    Local::now().time()
                }
            },
            duration: d,
            location: match location {
                Some(loc) => String::from(loc),
                None => String::from(""),
            },
            recurrence: match recurr {
                Some(val) => parse_recurrence(val),
                None => None,
            },
            metadata: match tags {
                Some(t) => EventMetadata {
                    tags: t,
                    creation: Local::now(),
                    modification: Local::now(),
                },
                None => EventMetadata::default(),
            },
        }
    }

    pub fn overlaps(&self, other: &Event) -> bool {
        let self_start = self.start_date.and_time(self.start_time);
        let other_start = other.get_start_date().and_time(other.get_start_time());
        let self_end = self_start + self.duration;
        let other_end = other_start + Duration::seconds(other.get_duration());
        let mut overlap = other_start <= self_end && other_end >= self_start;
        if overlap {
            overlap
        } else {
            if self.recurrence.is_some() {
                let rec = self.recurrence.as_ref().unwrap();
                let cad = rec.cadence();
                let cnt = rec.repetitions;
                for _ in 0..cnt {
                    let (new_start, new_end) = next_occurrence(&self, cad);
                    overlap = other_start <= new_start && other_end >= new_end;
                    if overlap {
                        return overlap;
                    }
                }
            }
            if other.get_recurrence().is_some() {
                let rec = self.recurrence.as_ref().unwrap();
                let cad = rec.cadence();
                let cnt = rec.repetitions;
                for _ in 0..cnt {
                    let (new_start, new_end) = next_occurrence(&other, cad);
                    overlap = new_start <= self_end && new_end >= self_start;
                    if overlap {
                        return overlap;
                    }
                }
            }
            false
        }
    }

    pub fn set_title(&mut self, new_title: &str) {
        self.title = String::from(new_title);
    }
    pub fn set_description(&mut self, new_descr: &str) {
        self.description = String::from(new_descr);
    }
    pub fn set_start_date(&mut self, d_m_y: (u32, u32, i32)) -> bool {
        match NaiveDate::from_ymd_opt(d_m_y.2, d_m_y.1, d_m_y.0) {
            Some(date) => {
                self.start_date = date;
                true
            }
            None => false,
        }
    }
    pub fn set_start_time(&mut self, hms: (u32, u32, u32)) -> bool {
        match NaiveTime::from_hms_opt(hms.0, hms.1, 0) {
            Some(time) => {
                self.start_time = time;
                true
            }
            None => false,
        }
    }
    pub fn set_duration(&mut self, new_duration: &Duration) {
        self.duration = Duration::to_owned(new_duration);
    }
    pub fn set_location(&mut self, loc: &str) {
        self.location = String::from(loc);
    }

    pub fn set_recurrence(&mut self, rec: &str) {
        self.recurrence = parse_recurrence(rec);
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.metadata.set_tags(tags);
    }

    pub fn get_title(&self) -> &str {
        self.title.as_str()
    }
    pub fn get_description(&self) -> &str {
        self.description.as_str()
    }
    pub fn get_start_date(&self) -> NaiveDate {
        self.start_date
    }
    pub fn get_start_time(&self) -> NaiveTime {
        self.start_time
    }
    /// returns the duration of this event, in seconds
    pub fn get_duration(&self) -> i64 {
        self.duration.num_seconds()
    }
    /// Returns the location of this event, if any
    pub fn get_location(&self) -> &str {
        self.location.as_str()
    }

    /// Returns the recurrence of this event, if any
    pub fn get_recurrence(&self) -> Option<&Recurrence> {
        self.recurrence.as_ref()
    }

    pub fn get_metadata(&self) -> EventMetadata {
        self.metadata.clone()
    }
}

impl Default for Event {
    fn default() -> Event {
        let now = Local::now();
        Event {
            title: String::new(),
            description: String::new(),
            start_date: now.date_naive(),
            start_time: now.time(),
            duration: Duration::zero(),
            location: String::from(""),
            recurrence: None,
            metadata: EventMetadata::default(),
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut h);
        let hashval = h.finish();

        let desc = self.get_description();
        let mut loc = String::from(self.get_location());
        if !loc.is_empty() {
            loc = " @ ".to_owned() + &loc;
        }
        write!(
            f,
            "[eid = {}]\n[{} - {}] {}{}\n{}",
            hashval,
            self.get_start_date().format("%d/%m/%Y"),
            self.get_start_time().format("%H:%M"),
            self.get_title(),
            &loc,
            if desc.len() < 50 {
                desc.to_string()
            } else {
                desc[0..49].to_string() + "..."
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{Cadence, Event, Recurrence};
    use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Timelike};

    #[test]
    /// tests the new function
    fn test_event_new() {
        let t = String::from("Some title");
        let des = String::from("Some description");
        let dt = NaiveDate::from_ymd_opt(2022, 7, 13).unwrap();
        let tm = NaiveTime::from_hms_opt(12, 23, 0).unwrap();
        let dur = 2.75;
        let loc = String::from("Some location");

        println!("{} {}", dt, tm);

        let e1 = Event::new(
            &t,
            &des,
            &dt.to_string(),
            &tm.to_string(),
            dur,
            Some(loc.as_str()),
            None,
            None,
        );
        let mut e2 = Event::default();
        assert_ne!(e1.title, e2.title);
        e2.set_title(&t);
        assert_eq!(e1.title, e2.title);
        assert_ne!(e1.description, e2.description);
        e2.set_description(&des);
        assert_eq!(e1.description, e2.description);
        assert_ne!(e1.start_date, e2.start_date);
        assert!(e2.set_start_date((dt.day(), dt.month(), dt.year())));
        assert_eq!(e1.start_date, e2.start_date);
        assert_ne!(e1.start_time, e2.start_time);
        assert!(e2.set_start_time((tm.hour(), tm.minute(), tm.second())));
        assert_eq!(e1.start_time, e2.start_time);
        assert_ne!(e1.duration, e2.duration);
        e2.set_duration(&Duration::hours(dur as i64));
        assert_eq!(e1.duration, e2.duration);
        assert_ne!(e1.location, e2.location);
        e2.set_location(loc.as_str());
        assert_eq!(e1.location, e2.location);
    }

    #[test]
    /// Tests all recognized date & time formats
    fn test_date_time_formats() {
        let test_date = "10/03/2022";
        let test_time = "16:10";
        let fmt_date = "%d/%m/%Y";
        let fmt_time = "%H:%M";
        let dmy_hm = Event::new("test", "test", test_date, test_time, 1.0, None, None, None);
        assert_eq!(
            dmy_hm.get_start_date(),
            chrono::NaiveDate::parse_from_str(test_date, fmt_date).unwrap()
        );
        assert_eq!(
            dmy_hm.get_start_time(),
            chrono::NaiveTime::parse_from_str(test_time, fmt_time).unwrap()
        );
    }

    #[test]
    /// Test recurrent events (secondly)
    fn test_recurrent_secondly() {
        // an event that repeats each second for 55 times
        let ev_min = Event::new(
            "test",
            "test",
            "xxx",
            "xxx",
            1.0,
            None,
            Some("minutely 55"),
            None,
        );
        assert_eq!(
            ev_min.get_recurrence(),
            Some(&Recurrence {
                cadence: Cadence::Minutely,
                repetitions: 55,
                ..Recurrence::default()
            })
        );
    }

    #[test]
    /// Test recurrent events (minutely)
    fn test_recurrent_minutely() {
        // an event that repeats each minute for 55 times
        let ev_sec = Event::new(
            "test",
            "test",
            "xxx",
            "xxx",
            1.0,
            None,
            Some("secondly 55"),
            None,
        );
        assert_eq!(
            ev_sec.get_recurrence(),
            Some(&Recurrence {
                cadence: Cadence::Secondly,
                repetitions: 55,
                ..Recurrence::default()
            })
        );
    }

    #[test]
    /// Test recurrent events (daily)
    fn test_recurrent_daily() {
        // an event that repeats daily for 5 days
        let ev_daily = Event::new(
            "test",
            "test",
            "xxx",
            "yyy",
            1.0,
            None,
            Some("daily 5"),
            None,
        );
        assert_eq!(
            ev_daily.get_recurrence(),
            Some(&Recurrence {
                cadence: Cadence::Daily,
                repetitions: 5,
                ..Recurrence::default()
            })
        );
    }

    #[test]
    /// Test recurrent events (weekly)
    fn test_recurrent_weekly() {
        // an event that repeats weekly for 2 weeks
        let ev_weekly = Event::new(
            "test",
            "test",
            "xxx",
            "yyy",
            1.0,
            None,
            Some("Weekly 2"),
            None,
        );
        assert_eq!(
            ev_weekly.get_recurrence(),
            Some(&Recurrence {
                cadence: Cadence::Weekly,
                repetitions: 2,
                ..Recurrence::default()
            })
        );
    }

    #[test]
    /// Test recurrent events (monthly)
    fn test_recurrent_monthly() {
        // an event that repeats monthly for 12 months
        let ev_monthly = Event::new(
            "test",
            "test",
            "xxx",
            "yyy",
            1.0,
            None,
            Some("MONTHLY 12"),
            None,
        );
        assert_eq!(
            ev_monthly.get_recurrence(),
            Some(&Recurrence {
                cadence: Cadence::Monthly,
                repetitions: 12,
                ..Recurrence::default()
            })
        );
    }

    #[test]
    /// Test recurrent events (invalid)
    fn test_recurrent_bad() {
        // an event that does not repeat (badly formatted)
        let ev_bad_fmt = Event::new(
            "test",
            "test",
            "xxx",
            "yyy",
            1.0,
            None,
            Some("Monthly -1"),
            None,
        );
        assert_eq!(ev_bad_fmt.get_recurrence(), None);
        // an event that repeats yearly for 110 years
        let ev_yearly = Event::new(
            "test",
            "test",
            "xxx",
            "yyy",
            1.0,
            None,
            Some("YearLY 110"),
            None,
        );
        assert_eq!(
            ev_yearly.get_recurrence(),
            Some(&Recurrence {
                cadence: Cadence::Yearly,
                repetitions: 110,
                ..Recurrence::default()
            })
        );
    }

    #[test]
    /// Test recurrent events (0 repeats)
    fn test_recurrent_zero() {
        // an events that repeats 0 times (does not repeat)
        let ev_zero_rep = Event::new(
            "test",
            "test",
            "xxx",
            "yyy",
            1.0,
            None,
            Some("daily 0"),
            None,
        );
        assert_eq!(ev_zero_rep.get_recurrence(), None);
    }
}
