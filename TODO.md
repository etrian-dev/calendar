# Calenda.rs
## Event struct
 - [x] Add location string (also in ics parsing)
 - [ ] fix integration tests in directory tests/
 - [ ] better handling of serialization/deserialization of calendars
## Subcommands
 - [Add event](#add)
 - [Remove event](#remove)
 - [List events](#list)
## Add
 - [x] from cli parameters
 - [x] **EXPERIMENTAL** from file (lookup iCalendar specification for .ics files)
## Remove
 - [x] Given event id, remove element
## List
Lists by default all events
 - [x] -t,--today filter
 - [x] -w, --week filter
 - [x] -m,--month filter
 - [ ] --from %d/%m/%yyyy,--until %d/%m/%yyyy filters
 - [ ] change default filter to --from-today