use std::fmt::{Debug, Display};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::calendar_error::CalendarError;
use crate::event::Event;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Calendar {
    name: String,
    events: Vec<Event>,
}

impl Calendar {
    pub fn new(calendar_name: &str) -> Calendar {
        Calendar {
            name: String::from(calendar_name),
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, ev: Event) {
        self.events.push(ev);
    }

    pub fn remove_event(&mut self, eid: u64) -> Result<Event, CalendarError> {
        let mut idx: usize = 0;
        for ev in &self.events {
            if ev.get_eid() == eid {
                return Ok(self.events.swap_remove(idx));
            }
            idx += 1;
        }
        Err(CalendarError::EventNotFound(eid))
    }

    pub fn list_events<F>(&self, filter: F) -> Vec<&Event>
    where
        F: Fn(&Vec<Event>) -> Vec<&Event>,
    {
        filter(&self.events)
    }
    pub fn list_events_today(&self) -> Vec<&Event> {
        let mut events_today = Vec::new();
        // get current date
        let curr_date = Local::today();
        for ev in &self.events {
            if ev.get_start_date() == curr_date {
                events_today.push(ev);
            }
        }
        events_today
    }
}

impl Display for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "--- {} ---\n# events: {}\n",
            self.name,
            self.events.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar::Calendar;
    use crate::event::Event;

    #[test]
    /// tests the event addition method
    fn test_event_addition() {
        let e1 = Event::default();
        let mut e2 = Event::default();
        e2.set_title("New title");

        let mut empty_cal = Calendar::new("test");
        let full_cal = Calendar {
            name: String::from("test"),
            events: vec![e1.clone(), e2.clone()],
        };

        empty_cal.add_event(e1);
        empty_cal.add_event(e2);

        assert_eq!(empty_cal, full_cal);
    }

    #[test]
    /// tests the event deletion method
    fn test_event_deletion() {
        let e = Event::default();
        let eid = e.get_eid();

        let mut cal = Calendar::new("test");
        cal.add_event(e);

        assert!(cal.remove_event(rand::random()).is_err());
        assert!(cal.remove_event(eid).is_ok());
        assert!(cal.remove_event(eid).is_err());
    }
}
