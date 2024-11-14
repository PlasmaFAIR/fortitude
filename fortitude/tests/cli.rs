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

#[test]
fn explain_all() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("explain");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("S061"));

    Ok(())
}

#[test]
fn explain_nonexistent_rule() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("explain").arg("X99999");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));

    Ok(())
}

#[test]
fn explain_one_rule() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("explain").arg("T001");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_one_rule_by_name() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("explain").arg("implicit-typing");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_category() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("explain").arg("T");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("T002"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_category_by_name() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("explain").arg("typing");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("T002"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}

#[test]
fn explain_mixed_multiple() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("fortitude")?;

    cmd.arg("explain").arg("T").arg("use-all").arg("P021");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("T001"))
        .stdout(predicate::str::contains("T002"))
        .stdout(predicate::str::contains("M011"))
        .stdout(predicate::str::contains("P021"))
        .stdout(predicate::str::contains("S061").count(0));

    Ok(())
}
