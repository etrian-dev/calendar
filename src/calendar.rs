use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::calendar_error::CalendarError;
use crate::event::{Cadence, Event, Recurrence};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Calendar {
    owner: String,
    name: String,
    events: HashMap<u64, Event>,
}

/// Given a recurrence and starting date and time, computes the dates and times
/// of the recurrences of the event and returns them as a vector
fn expand_recurrence(rec: &Recurrence, dt: &NaiveDate, tm: &NaiveTime) -> Vec<NaiveDateTime> {
    let mut rec_dates = Vec::new();
    for i in 0..=rec.repetitions() {
        let x = NaiveDateTime::new(*dt, *tm);
        let dt_new = match rec.cadence() {
            Cadence::Secondly => x + Duration::seconds(i as i64),
            Cadence::Minutely => x + Duration::minutes(i as i64),
            Cadence::Hourly => x.checked_add_signed(Duration::hours(i as i64)).unwrap(),
            Cadence::Daily => x.checked_add_signed(Duration::days(i as i64)).unwrap(),
            Cadence::Weekly => x.checked_add_signed(Duration::weeks(i as i64)).unwrap(),
            Cadence::Monthly => x.with_month(dt.month() + i as u32).unwrap(),
            Cadence::Yearly => x.with_year(dt.year() + i as i32).unwrap(),
        };
        rec_dates.push(dt_new);
    }
    rec_dates
}

impl Calendar {
    pub fn new(owner_name: &str, calendar_name: &str) -> Calendar {
        Calendar {
            owner: String::from(owner_name),
            name: String::from(calendar_name),
            events: HashMap::new(),
        }
    }

    pub fn get_owner(&self) -> &str {
        &self.owner
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_size(&self) -> usize {
        self.events.len()
    }

    pub fn set_owner(&mut self, s: &str) {
        self.owner = String::from(s);
    }

    pub fn set_name(&mut self, s: &str) {
        self.name = String::from(s);
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn get_event(&mut self, eid: u64) -> Result<&mut Event, CalendarError> {
        if let Some(ev) = self.events.get_mut(&eid) {
            Ok(ev)
        } else {
            Err(CalendarError::EventNotFound(eid))
        }
    }

    pub fn add_event(&mut self, ev: Event) -> bool {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        ev.hash(&mut h);
        let ev_hash = h.finish();
        if self.events.contains_key(&ev_hash) {
            warn!(
                "Event with hash {} already in this calendar: calendar not modified",
                ev_hash
            );
            eprintln!(
                "Event \"{}\" already in this calendar: calendar not modified",
                ev.get_title()
            );
            return false;
        }
        // Warn the user if this event overlaps with some other event
        for e in self.events.values() {
            if e.overlaps(&ev) {
                e.hash(&mut h);
                let e_hash = h.finish();
                warn!(
                    "Warning: the event {} overlaps with event {}",
                    ev_hash, e_hash
                );
                eprintln!(
                    "Warning: the event \"{}\" overlaps with event \"{}\"",
                    ev.get_title(),
                    e.get_title()
                );
            }
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

    /// TODO: provide some helpers like before
    pub fn list_events_between(
        &self,
        from: Option<NaiveDateTime>,
        until: Option<NaiveDateTime>,
    ) -> Vec<Event> {
        let mut events_between = Vec::new();
        let from_dt = from.unwrap_or(NaiveDateTime::MIN);
        let until_dt = until.unwrap_or(NaiveDateTime::MAX);

        for ev in self.events.values() {
            let ev_dt = ev.get_start_date().and_time(ev.get_start_time());
            // If the event is recurrent then expand its recurrent dates
            // if any of those is equal to the current then add the modified event to output vec
            if let Some(rec) = ev.get_recurrence() {
                for rec_dt in expand_recurrence(rec, &ev.get_start_date(), &ev.get_start_time()) {
                    if rec_dt >= from_dt && rec_dt <= until_dt {
                        // Since cloning is expensive it is done only on recurrences that should appear
                        // in the output vector
                        let mut ev2 = ev.clone();
                        ev2.set_start_date((rec_dt.day(), rec_dt.month(), rec_dt.year()));
                        ev2.set_start_time((rec_dt.hour(), rec_dt.minute(), rec_dt.second()));
                        events_between.push(ev2);
                    }
                }
            } else if ev_dt <= until_dt && ev_dt >= from_dt {
                events_between.push(ev.clone());
            }
        }
        // sorts events by their start date and then start time
        events_between.sort_unstable_by(|e1, e2| {
            if e1.get_start_date().cmp(&e2.get_start_date()) == core::cmp::Ordering::Equal {
                e1.get_start_time().cmp(&e2.get_start_time())
            } else {
                e1.get_start_date().cmp(&e2.get_start_date())
            }
        });
        events_between
    }

    pub fn list_events_tagged(&self, tag: String) -> Vec<Event> {
        let mut filtered_events = Vec::new();
        for ev in self.events.values() {
            let ev_tags = ev.get_metadata().get_tags();
            if ev_tags.contains(&tag) {
                filtered_events.push(ev.clone());
            }
        }
        filtered_events
    }
}

impl Display for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tot_events = 0;
        for ev in self.events.values() {
            if let Some(rec) = ev.get_recurrence() {
                tot_events += rec.repetitions() + 1;
            } else {
                tot_events += 1;
            }
        }
        write!(
            f,
            "--- {} ({}) ---\ntotal events: {}\n{}",
            self.name,
            self.owner,
            tot_events,
            Local::now().format("%A %d/%m/%Y - %H:%M")
        )
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Calendar {
            owner: String::from("default"),
            name: String::from("default"),
            events: HashMap::new(),
        }
    }
}
#[cfg(test)]
mod tests {
    use chrono::{Datelike, Local, Timelike};
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
        let e2 = Event::default();
        let e1_hash = get_hash(&e1);
        let e2_hash = get_hash(&e2);

        let mut empty_cal = Calendar::new("owner", "test");
        let full_cal = Calendar {
            owner: String::from("owner"),
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
        // defines some events
        let v = vec![
            Event::new("title1", "desc1", "11/02/2001", "-", 3.6, None, None, None),
            Event::new(
                "title2",
                "desc2",
                "08/09/2011",
                "-",
                3.6,
                Some("Some location"),
                None,
                None,
            ),
            Event::new(
                "title3",
                "desc3",
                "11/02/2001",
                "-",
                3.6,
                Some("Random loc"),
                None,
                None,
            ),
            Event::new("title4", "desc4", "13/04/1999", "-", 3.6, None, None, None),
            Event::new("title5", "desc5", "21/01/2021", "-", 3.6, None, None, None),
            Event::new("title6", "desc6", "13/03/2001", "-", 3.6, None, None, None),
            Event::new(
                "title7",
                "desc7",
                "12/12/2012",
                "-",
                3.6,
                Some("Pisa"),
                None,
                None,
            ),
        ];

        let mut cal = Calendar::new("owner", "test_multiple_cal");
        assert_eq!(cal.events.len(), 0);
        for ev in v.clone() {
            cal.add_event(ev);
        }
        assert_eq!(cal.events.len(), v.len());

        for ev in &v {
            let h = get_hash(ev);
            assert!(cal.events.contains_key(&h));
        }
    }

    #[test]
    /// tests the event deletion method
    fn test_event_deletion() {
        let e = Event::default();
        let eid = get_hash(&e);

        let mut cal = Calendar::new("owner", "test");
        cal.add_event(e);

        assert!(cal.remove_event(rand::random()).is_err());
        assert!(cal.remove_event(eid).is_ok());
        assert!(cal.remove_event(eid).is_err());
    }

    #[test]
    /// test week filter
    fn test_week_filter() {
        let dt = Local::now().naive_local();
        let mut cal = Calendar::new("owner", "test");
        for offt in -365..365 {
            let date_offt = dt.checked_add_signed(chrono::Duration::days(offt)).unwrap();
            let e = event::Event::new(
                "test",
                "test",
                &date_offt.to_string(),
                &dt.time().format("%H:%M").to_string(),
                1.0,
                None,
                None,
                None,
            );
            cal.add_event(e);
        }

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
        for ev in cal.list_events_between(Some(start), Some(end)) {
            assert_eq!(ev.get_start_date().iso_week(), dt.iso_week());
        }
    }

    #[test]
    /// tests that duplicate events (events with the same hash) are not added
    fn test_duplicate_add() {
        let mut cal = Calendar::new("owner", "test");
        let ev = Event::new(
            "title",
            "description",
            "10/02/2011",
            "15:00",
            4.2,
            Some("Somewhere"),
            None,
            None,
        );
        assert_eq!(cal.events.len(), 0);
        cal.add_event(ev.clone());
        assert_eq!(cal.events.len(), 1);
        // trying to add an event with the same hash should not result in a new event being added
        cal.add_event(ev.clone());
        assert_eq!(cal.events.len(), 1);
        let mut ev2 = ev;
        // but if the event is mutated than it should have a different hash and hence be added
        ev2.set_title("Random");
        cal.add_event(ev2);
        assert_eq!(cal.events.len(), 2);
    }

    #[test]
    /// tests the clear method
    fn test_clear() {
        // defines some events
        let v = vec![
            Event::new("title1", "desc1", "11/02/2001", "-", 3.6, None, None, None),
            Event::new(
                "title2",
                "desc2",
                "08/09/2011",
                "-",
                3.6,
                Some("Some location"),
                None,
                None,
            ),
            Event::new(
                "title3",
                "desc3",
                "11/02/2001",
                "-",
                3.6,
                Some("Random loc"),
                None,
                None,
            ),
            Event::new("title4", "desc4", "13/04/1999", "-", 3.6, None, None, None),
            Event::new("title5", "desc5", "21/01/2021", "-", 3.6, None, None, None),
            Event::new("title6", "desc6", "13/03/2001", "-", 3.6, None, None, None),
            Event::new(
                "title7",
                "desc7",
                "12/12/2012",
                "-",
                3.6,
                Some("Pisa"),
                None,
                None,
            ),
        ];
        let mut cal = Calendar::new("owner", "test");
        let vlen = v.len();
        for ev in v {
            cal.add_event(ev);
        }
        assert_eq!(vlen, cal.list_events_between(None, None).len());
        cal.clear();
        assert_eq!(0, cal.list_events_between(None, None).len());
    }
}
