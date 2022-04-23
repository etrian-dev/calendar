# Calenda.rs
## Event struct
 - [x] Add location string (also in ics parsing)
 - [ ] fix integration tests in directory tests/
 - [ ] better handling of serialization/deserialization of calendars
 - [x] handle --create, --view and --delete flags
 - [ ] a read-only mode for calendars
 - [ ] flag to --list all known calendars in ./data
## Subcommands
 - [Add event](#add)
 - [Remove event](#remove)
 - [List events](#list)
## Add
 - [x] from cli parameters
 - [x] **EXPERIMENTAL** from file (lookup iCalendar specification for .ics files)
## Remove
 - [x] Given event hash, remove element
## List
Lists by default all events
 - [x] -t,--today filter
 - [x] -w, --week filter
 - [x] -m,--month filter
 - [x] --from %d/%m/%yyyy,--until %d/%m/%yyyy filters (they actually implement the identity filter)
 - [x] change default filter to --from [current date]
 - [ ] provide generic, user-definable filter