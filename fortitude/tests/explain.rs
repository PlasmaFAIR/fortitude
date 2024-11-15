use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const BIN_NAME: &str = "fortitude";

#[test]
fn explain_all() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("S061"));

    Ok(())
}

#[test]
fn explain_nonexistent_rule() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("X99999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));

    Ok(())
}

#[test]
fn explain_one_rule() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("T001")
        .assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_one_rule_by_name() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("implicit-typing")
        .assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_category() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("T")
        .assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("T002"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_category_by_name() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("typing")
        .assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("T002"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_mixed_multiple() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("T")
        .arg("use-all")
        .arg("P021")
        .assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("T002"))
        .stdout(predicate::str::contains("M011"))
        .stdout(predicate::str::contains("P021"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}
