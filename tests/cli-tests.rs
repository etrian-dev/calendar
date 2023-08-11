use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

/*
#[test]
fn add_event() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: common harness create calendar
    let mut cmd = Command::cargo_bin("calendar")?;
    cmd.args(["-c", "test"]);
    cmd.assert().success();
    cmd.args([
        "-e",
        "test",
        "add",
        "some title",
        "some description",
        "11/03/2022",
        "11:17",
        "2.5",
    ]);
    cmd.assert().success();
    Ok(())
}
*/

/*#[test]
fn add_from_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("calendar")?;
    cmd.args(["add", "--from-file", "2_events_ok.ics"]);
    let out = cmd.output().expect("failed adding events from ics file");
    let outstr = std::str::from_utf8(&out.stdout)?;
    println!("{}", outstr);
    cmd.assert().to_string().contains("2 events");
    Ok(())
}*/
