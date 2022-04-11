use chrono::{Duration, Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::vec;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    eid: u64,
    title: String,
    description: String,
    start_date: NaiveDate,
    start_time: NaiveTime,
    #[serde(serialize_with = "duration_to_min")]
    #[serde(deserialize_with = "min_to_duration")]
    duration: Duration,
    location: String,
}

impl Event {
    pub fn new(
        event_title: &str,
        descr: &str,
        start_date: &str,
        start_time: &str,
        dur: f32,
        location: Option<&str>,
    ) -> Event {
        let date_formats = vec!["%d/%m/%Y", "%Y-%m-%d"];
        let mut date = Err(());
        for fmt in date_formats {
            if let Ok(val) = NaiveDate::parse_from_str(start_date, fmt) {
                date = Ok(val);
                break;
            } // else wrong format
        }

        let time_formats = vec!["%H:%M", "%H:%M:%S"];
        let mut time = Err(());
        for fmt in time_formats {
            if let Ok(val) = NaiveTime::parse_from_str(start_time, fmt) {
                time = Ok(val);
                break;
            } // else wrong format
        }

        let d = Duration::hours((dur as i32).into());
        Event {
            // add a unique, random, event id
            eid: rand::random(),
            title: event_title.to_string(),
            description: descr.to_string(),
            start_date: match date {
                Ok(date) => date,
                Err(_) => Local::now().date().naive_local(),
            },
            start_time: match time {
                Ok(tm) => tm,
                Err(_) => Local::now().time(),
            },
            duration: d,
            location: match location {
                Some(loc) => String::from(loc),
                None => String::from(""),
            },
        }
    }

    pub fn set_title(&mut self, new_title: &str) {
        self.title = String::from(new_title);
    }
    pub fn set_description(&mut self, new_descr: &str) {
        self.description = String::from(new_descr);
    }
    pub fn set_start_date(&mut self, d_m_y: (u32, u32, i32)) {
        self.start_date = NaiveDate::from_ymd(d_m_y.2, d_m_y.1, d_m_y.0);
    }
    pub fn set_start_time(&mut self, hms: (u32, u32, u32)) {
        self.start_time = NaiveTime::from_hms(hms.0, hms.1, 0);
    }
    pub fn set_duration(&mut self, new_duration: &Duration) {
        self.duration = Duration::to_owned(new_duration);
    }
    pub fn set_location(&mut self, loc: &str) {
        self.location = String::from(loc);
    }

    pub fn get_eid(&self) -> u64 {
        self.eid
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
}

impl Default for Event {
    fn default() -> Event {
        let now = Local::now();
        Event {
            eid: rand::random(),
            title: String::new(),
            description: String::new(),
            start_date: now.date().naive_local(),
            start_time: now.time(),
            duration: Duration::zero(),
            location: String::from(""),
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = self.get_description();
        let idx = if desc.len().min(49) > 0 {
            desc.len().min(49)
        } else {
            0
        };
        let mut loc = String::from(self.get_location());
        if loc.len() > 0 {
            loc = " @ ".to_owned() + &loc;
        }
        write!(
            f,
            "[{} - {}] {}{}\n{}",
            self.get_start_date().format("%d/%m/%Y"),
            self.get_start_time().format("%H:%M"),
            self.get_title(),
            &loc,
            desc.get(0..idx).unwrap_or("Failed to get description")
        )
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.eid == other.eid
    }

    fn ne(&self, other: &Event) -> bool {
        !self.eq(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::event::Event;
    use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Timelike};

    #[test]
    /// tests the new function
    fn test_event_new() {
        let t = String::from("Some title");
        let des = String::from("Some description");
        let dt = NaiveDate::from_ymd(2022, 7, 13);
        let tm = NaiveTime::from_hms(12, 23, 0);
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
        );
        let mut e2 = Event::default();
        assert_ne!(e1.title, e2.title);
        e2.set_title(&t);
        assert_eq!(e1.title, e2.title);
        assert_ne!(e1.description, e2.description);
        e2.set_description(&des);
        assert_eq!(e1.description, e2.description);
        assert_ne!(e1.start_date, e2.start_date);
        e2.set_start_date((dt.day(), dt.month(), dt.year()));
        assert_eq!(e1.start_date, e2.start_date);
        assert_ne!(e1.start_time, e2.start_time);
        e2.set_start_time((tm.hour(), tm.minute(), tm.second()));
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
        let dmy_hm = Event::new("test", "test", test_date, test_time, 1.0, None);
        assert_eq!(
            dmy_hm.get_start_date(),
            chrono::NaiveDate::parse_from_str(test_date, fmt_date).unwrap()
        );
        assert_eq!(
            dmy_hm.get_start_time(),
            chrono::NaiveTime::parse_from_str(test_time, fmt_time).unwrap()
        );
    }
}
