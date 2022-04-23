use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::calendar_error::CalendarError;
use crate::event::Event;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Calendar {
    name: String,
    events: HashMap<u64, Event>,
}

impl Calendar {
    pub fn new(calendar_name: &str) -> Calendar {
        Calendar {
            name: String::from(calendar_name),
            events: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_event(&mut self, ev: Event) -> bool {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        ev.hash(&mut h);
        let ev_hash = h.finish();
        dbg!(ev_hash);
        dbg!(self.events.keys());
        if self.events.contains_key(&ev_hash) {
            return false;
        }
        self.events.insert(ev_hash, ev);
        true
    }

    /// Removes an event, given its hash
    pub fn remove_event(&mut self, eid: u64) -> Result<Event, CalendarError> {
        match self.events.remove(&eid) {
            Some(event) => Ok(event),
            None => Err(CalendarError::EventNotFound(eid)),
        }
    }

    pub fn list_events_today(&self) -> Vec<&Event> {
        let mut events_today = Vec::new();
        // get current date
        let curr_date = Local::today().naive_local();
        for ev in self.events.values() {
            if curr_date == ev.get_start_date() {
                events_today.push(ev);
            }
        }
        events_today
    }
    pub fn list_events_week(&self) -> Vec<&Event> {
        let mut events_week = Vec::new();
        // get current date
        let week = Local::today();

        for ev in self.events.values() {
            if ev.get_start_date().iso_week() == week.iso_week() {
                events_week.push(ev);
            }
        }
        events_week
    }

    pub fn list_events_month(&self) -> Vec<&Event> {
        let mut events_month = Vec::new();
        // get current date
        let dt = Local::today();
        let curr_month = dt.month();
        let curr_year = dt.year();

        for ev in self.events.values() {
            if ev.get_start_date().month() == curr_month && ev.get_start_date().year() == curr_year
            {
                events_month.push(ev);
            }
        }
        events_month
    }

    pub fn list_events_between(&self, from: Option<String>, until: Option<String>) -> Vec<&Event> {
        let from_date = match from {
            Some(s) => NaiveDate::parse_from_str(&s, "%d/%m/%Y").unwrap_or(chrono::naive::MIN_DATE),
            None => chrono::naive::MIN_DATE,
        };
        let until_date = match until {
            Some(s) => NaiveDate::parse_from_str(&s, "%d/%m/%Y").unwrap_or(chrono::naive::MAX_DATE),
            None => chrono::naive::MAX_DATE,
        };

        let mut events_between = Vec::new();
        for ev in self.events.values() {
            if ev.get_start_date() <= until_date && ev.get_start_date() >= from_date {
                events_between.push(ev);
            }
        }
        events_between
    }
}

impl Display for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "--- Calendar: {} ---\n# events: {}",
            self.name,
            self.events.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    use crate::calendar::Calendar;
    use crate::event::{self, Event};

    fn get_hash(e: &Event) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        e.hash(&mut h);
        h.finish()
    }

    #[test]
    /// tests the event addition method
    fn test_event_addition() {
        let e1 = Event::default();
        let mut e2 = Event::default();
        let e1_hash = get_hash(&e1);
        let e2_hash = get_hash(&e2);
        e2.set_title("New title");

        let mut empty_cal = Calendar::new("test");
        let full_cal = Calendar {
            name: String::from("test"),
            events: HashMap::from([(e1_hash, e1.clone()), (e2_hash, e2.clone())]),
        };

        empty_cal.add_event(e1);
        empty_cal.add_event(e2);

        assert_eq!(empty_cal, full_cal);
    }

    #[test]
    /// tests adding multiple different events
    fn test_event_multiple() {
        use std::iter::zip;
        // defines some events
        let v = vec![
            Event::new("title1", "desc1", "11/02/2001", "-", 3.6, None),
            Event::new(
                "title2",
                "desc2",
                "08/09/2011",
                "-",
                3.6,
                Some("Some location"),
            ),
            Event::new(
                "title3",
                "desc3",
                "11/02/2001",
                "-",
                3.6,
                Some("Random loc"),
            ),
            Event::new("title4", "desc4", "13/04/1999", "-", 3.6, None),
            Event::new("title5", "desc5", "21/01/2021", "-", 3.6, None),
            Event::new("title6", "desc6", "13/03/2001", "-", 3.6, None),
            Event::new("title7", "desc7", "12/12/2012", "-", 3.6, Some("Pisa")),
        ];

        let mut cal = Calendar::new("test_multiple_cal");
        for ev in v.clone() {
            cal.add_event(ev);
        }

        // The identity filter is just implemented with None args in the method call below
        for ev in zip(cal.list_events_between(None, None), &v) {
            assert_eq!(ev.0, ev.1);
        }
    }

    #[test]
    /// tests the event deletion method
    fn test_event_deletion() {
        let e = Event::default();
        let eid = get_hash(&e);

        let mut cal = Calendar::new("test");
        cal.add_event(e);

        assert!(cal.remove_event(rand::random()).is_err());
        assert!(cal.remove_event(eid).is_ok());
        assert!(cal.remove_event(eid).is_err());
    }

    #[test]
    /// test week filter
    fn test_week_filter() {
        let dt = chrono::Local::now();
        let mut cal = Calendar::new("test");
        for offt in -365..365 {
            let date_offt = dt
                .date()
                .checked_add_signed(chrono::Duration::days(offt))
                .unwrap();
            let e = event::Event::new(
                "test",
                "test",
                &date_offt.to_string(),
                &dt.time().format("%H:%M").to_string(),
                1.0,
                None,
            );
            cal.add_event(e);
        }

        for ev in cal.list_events_week() {
            assert_eq!(ev.get_start_date().iso_week(), dt.iso_week());
        }
    }
}
