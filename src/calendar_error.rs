use std::fmt;
use std::fmt::{Debug, Display};

pub enum CalendarError {
    EventNotFound(u64),
}

impl Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarError::EventNotFound(_) => write!(f, "Event not found!"),
        }
    }
}
impl Debug for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CalendarError::EventNotFound(eid) => write!(f, "Event {} not found!", eid),
        }
    }
}
