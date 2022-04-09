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
}

impl Event {
    pub fn new(
        event_title: &str,
        descr: &str,
        start_date: &str,
        start_time: &str,
        dur: f32,
    ) -> Event {
        let date_formats = vec!["%d/%m/%Y"];
        let mut date = Err(());
        for fmt in date_formats {
            if let Ok(val) = NaiveDate::parse_from_str(start_date, fmt) {
                date = Ok(val);
                break;
            } // else wrong format
        }

        let time_formats = vec!["%H:%M"];
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
}

impl Default for Event {
    fn default() -> Event {
        let now = Local::now();
        Event {
            eid: 0,
            title: String::new(),
            description: String::new(),
            start_date: now.date().naive_local(),
            start_time: now.time(),
            duration: Duration::zero(),
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} - {}] {}: {}",
            self.get_start_date().format("%d/%m/%Y"),
            self.get_start_time().format("%H:%M"),
            self.get_title(),
            String::from(self.get_description()) + ".."
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
    use chrono::{Datelike, Duration, Local, NaiveTime, TimeZone, Timelike};

    #[test]
    /// tests the new function
    fn test_event_new() {
        let t = String::from("Some title");
        let des = String::from("Some description");
        let dt = Local.ymd(2022, 03, 31);
        let tm = NaiveTime::from_hms(12, 23, 0);
        let dur = Duration::hours(2);
        let e1 = Event {
            eid: rand::random(),
            title: t.clone(),
            description: des.clone(),
            start_date: dt.naive_local(),
            start_time: tm,
            duration: dur,
        };
        let mut e2 = Event::default();
        e2.set_title(&t);
        assert_eq!(e1.title, e2.title);
        e2.set_description(&des);
        assert_eq!(e1.description, e2.description);
        e2.set_start_date((dt.day(), dt.month(), dt.year()));
        e2.set_start_time((tm.hour(), tm.minute(), tm.second()));
        assert_eq!(e1.start_date, e2.start_date);
        e2.set_duration(&dur);
        assert_eq!(e1.duration, e2.duration);
    }

    #[test]
    /// Tests all recognized date & time formats
    fn test_date_time_formats() {
        let test_date = "10/03/2022";
        let test_time = "16:10";
        let fmt_date = "%d/%m/%Y";
        let fmt_time = "%H:%M";
        let dmy_hm = Event::new("test", "test", test_date, test_time, 1.0);
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
