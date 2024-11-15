use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn check_file_doesnt_exist() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("check").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("No such file"));

    Ok(())
}
