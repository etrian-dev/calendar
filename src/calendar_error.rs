use std::fmt;
use std::fmt::{Debug, Display};
use std::io::Error;

#[derive(Clone)]
pub enum CalendarError {
    CalendarNotFound(String),
    CalendarAlreadyExists(String),
    EventNotFound(u64),
    IcsParsingFailed(String),
    Unknown(String),
}

impl Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CalendarNotFound(_) => write!(f, "Calendar not found"),
            Self::CalendarAlreadyExists(_) => write!(f, "The calendar already exists"),
            Self::EventNotFound(_) => write!(f, "Event not found!"),
            Self::IcsParsingFailed(_) => write!(f, "Failed parsing .ics file"),
            Self::Unknown(s) => write!(f, "Unknown error: {s}"),
        }
    }
}
impl Debug for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CalendarNotFound(s) => write!(f, "Calendar {s} not found"),
            Self::CalendarAlreadyExists(s) => write!(f, "Calendar {s} already exists"),
            Self::EventNotFound(eid) => write!(f, "Event {} not found!", eid),
            Self::IcsParsingFailed(file) => write!(f, "Failed parsing {file}"),
            Self::Unknown(s) => write!(f, "Unknown error: {s}"),
        }
    }
}
impl From<Error> for CalendarError {
    fn from(e: Error) -> Self {
        Self::CalendarNotFound(format!("Calendar not found: {}", e))
    }
}

