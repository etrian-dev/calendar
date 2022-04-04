use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn add_event() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("calendar")?;
    cmd.args([
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
