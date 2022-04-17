use std::fmt;
use std::fmt::{Debug, Display};

pub enum CalendarError {
    EventNotFound(u64),
    IcsParsingFailed(String),
}

impl Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarError::EventNotFound(_) => write!(f, "Event not found!"),
            CalendarError::IcsParsingFailed(_) => write!(f, "Failed parsing .ics file"),
        }
    }
}
impl Debug for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CalendarError::EventNotFound(eid) => write!(f, "Event {} not found!", eid),
            CalendarError::IcsParsingFailed(file) => write!(f, "Failed parsing {file}"),
        }
    }
}
