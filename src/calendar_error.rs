use std::fmt;
use std::fmt::{Debug, Display};
use std::io::Error;

pub enum CalendarError {
    CalendarNotFound(String),
    CalendarAlreadyExists(String),
    EventNotFound(u64),
    IcsParsingFailed(String),
}

impl Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarError::CalendarNotFound(_) => write!(f, "Calendar not found"),
            CalendarError::CalendarAlreadyExists(_) => write!(f, "The calendar already exists"),
            CalendarError::EventNotFound(_) => write!(f, "Event not found!"),
            CalendarError::IcsParsingFailed(_) => write!(f, "Failed parsing .ics file"),
        }
    }
}
impl Debug for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CalendarError::CalendarNotFound(s) => write!(f, "Calendar {s} not found"),
            CalendarError::CalendarAlreadyExists(s) => write!(f, "Calendar {s} already exists"),
            CalendarError::EventNotFound(eid) => write!(f, "Event {} not found!", eid),
            CalendarError::IcsParsingFailed(file) => write!(f, "Failed parsing {file}"),
        }
    }
}
impl From<Error> for CalendarError {
    fn from(e: Error) -> Self {
        CalendarError::CalendarNotFound(
            format!("Calendar not found: {}", e.to_string()))
    }
}
