# Calenda.rs
## Features
 - [ ] generation of .ics events from shell mode (needs support from the library)
 - [ ] add INTERVAL=?? in recurrence parsing (see [RFC](https://icalendar.org/iCalendar-RFC-5545/3-8-5-3-recurrence-rule.html))
 - [ ] detect and warn the user when adding overlapping events
 - [ ] shell mode as a binary
 - [ ] calendar owner (at creation and editing w/ flags)
 - [ ] test recurrence overlaps
## Event struct
 - [x] Add support for recurrent events
 - [ ] Support EXDATE property to exclude specific dates from RRULE
 - [x] Add location string (also in ics parsing)
 - [ ] fix integration tests in directory tests/
 - [ ] better handling of serialization/deserialization of calendars
 - [x] handle --create, --view and --delete flags
 - [x] handle --edit flag
 - [x] a read-only mode for calendars (--view)
 - [x] flag to --list all known calendars in ./data
## Subcommands
 - [Add event](#add)
 - [Remove event](#remove)
 - [List events](#list)
 - [Show event](#show)
## Add
 - [x] from cli parameters
 - [x] **EXPERIMENTAL** from file (lookup iCalendar specification for .ics files)
## Remove
 - [x] Given event hash, remove element
 - [ ] Support filter functions for remotion
 - [x] Flag -a|--all to remove all events
## List
Lists by default all events
 - [ ] Improve the Display trait impl for Calendar/Event
 - [x] -t,--today filter
 - [x] -w, --week filter
 - [x] -m,--month filter
 - [x] --from %d/%m/%yyyy,--until %d/%m/%yyyy filters
 - [x] change default filter to --from <current date>
 - [ ] provide generic, user-definable filter
 - [ ] Support regex matching of events to be listed
 - [ ] Improve the list_events_* family of functions: a lot of duplication
## Show
 - [ ] Show all the information on a specific event, given its eid