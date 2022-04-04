use chrono::{Date, DateTime, Duration, Local, NaiveTime, TimeZone, Timelike};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::vec;

fn datetime_to_ts<S>(dt: &DateTime<Local>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_i64(dt.timestamp())
}
fn duration_to_min<S>(dur: &Duration, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_i64(dur.num_minutes())
}

fn ts_to_datetime<'de, D>(de: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let x = i64::deserialize(de);
    match x {
        Ok(val) => Ok(Local.timestamp(val, 0)),
        Err(e) => Err(e),
    }
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
    #[serde(serialize_with = "datetime_to_ts")]
    #[serde(deserialize_with = "ts_to_datetime")]
    start_date: DateTime<Local>,
    #[serde(serialize_with = "duration_to_min")]
    #[serde(deserialize_with = "min_to_duration")]
    duration: Duration,
}

impl Default for Event {
    fn default() -> Event {
        Event {
            eid: 0,
            title: String::new(),
            description: String::new(),
            start_date: Local::now(),
            duration: Duration::zero(),
        }
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

impl Event {
    pub fn new(event_title: &str, descr: &str, start: &str, dur: f32) -> Event {
        let formats = vec!["%d/%m/%Y %R", "%F %T %:z", "%+"];
        let datetm = None;
        for fmt in formats {
            let datetm = match Local.datetime_from_str(start, fmt) {
                Ok(val) => Some(val),
                _ => None,
            };
            if datetm.is_some() {
                break;
            }
        }

        let d = Duration::hours((dur as i32).into());
        Event {
            // add a unique, random, event id
            eid: rand::random(),
            title: event_title.to_string(),
            description: descr.to_string(),
            start_date: match datetm {
                Some(date) => date,
                None => Local::now(),
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
        self.start_date = Local
            .ymd(d_m_y.2, d_m_y.1, d_m_y.0)
            .and_time(self.start_date.time())
            .unwrap();
    }
    pub fn set_start_time(&mut self, hms: (u32, u32, u32)) {
        self.start_date = self
            .start_date
            .with_hour(hms.0)
            .unwrap()
            .with_minute(hms.1)
            .unwrap()
            .with_second(hms.2)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
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
    pub fn get_start_date(&self) -> Date<Local> {
        self.start_date.date()
    }
    pub fn get_start_time(&self) -> NaiveTime {
        self.start_date.time()
    }
    /// returns the duration of this event, in seconds
    pub fn get_duration(&self) -> i64 {
        self.duration.num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use crate::event::Event;
    use chrono::{Datelike, Duration, Local, TimeZone, Timelike};

    #[test]
    /// tests the new function
    fn test_event_new() {
        let t = String::from("Some title");
        let des = String::from("Some description");
        let dt = Local.ymd(2022, 03, 31).and_hms(11, 2, 0);
        let dur = Duration::hours(2);
        let e1 = Event {
            eid: rand::random(),
            title: t.clone(),
            description: des.clone(),
            start_date: dt,
            duration: dur,
        };
        let mut e2 = Event::default();
        e2.set_title(&t);
        assert_eq!(e1.title, e2.title);
        e2.set_description(&des);
        assert_eq!(e1.description, e2.description);
        e2.set_start_date((dt.day(), dt.month(), dt.year()));
        e2.set_start_time((dt.hour(), dt.minute(), dt.second()));
        assert_eq!(e1.start_date, e2.start_date);
        e2.set_duration(&dur);
        assert_eq!(e1.duration, e2.duration);
    }
}
