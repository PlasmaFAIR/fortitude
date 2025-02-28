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
        .stdout(predicate::str::contains("C001"))
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
        .arg("C001")
        .assert()
        .success()
        .stdout(predicate::str::contains("C001"))
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
        .stdout(predicate::str::contains("C001"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_category() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("C")
        .assert()
        .success()
        .stdout(predicate::str::contains("C001"))
        .stdout(predicate::str::contains("C002"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_category_by_name() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("correctness")
        .assert()
        .success()
        .stdout(predicate::str::contains("C001"))
        .stdout(predicate::str::contains("C002"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_mixed_multiple() -> anyhow::Result<()> {
    Command::cargo_bin(BIN_NAME)?
        .arg("explain")
        .arg("OB")
        .arg("use-all")
        .arg("S201")
        .assert()
        .success()
        .stdout(predicate::str::contains("OB011"))
        .stdout(predicate::str::contains("C121"))
        .stdout(predicate::str::contains("S201"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}
