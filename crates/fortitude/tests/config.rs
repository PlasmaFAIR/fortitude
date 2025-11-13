//! Tests for the `fortitude config` subcommand.
use std::process::Command;

use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};

const BIN_NAME: &str = "fortitude";

#[test]
fn check_select() {
    assert_cmd_snapshot!(
        Command::new(get_cargo_bin(BIN_NAME)).arg("config").arg("check.select"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    A list of rule codes or prefixes to enable. Prefixes can specify exact
    rules (like `T003` or `superfluous-implicit-none`), entire categories
    (like `T` or `typing`), or anything in between.

    When breaking ties between enabled and disabled rules (via `select` and
    `ignore`, respectively), more specific prefixes override less
    specific prefixes.

    Default value: ["E", "F", "S", "T", "OB", "P", "M", "IO", "R", "B"]
    Type: list[RuleSelector]
    Example usage:
    ```toml
    # Only check errors and obsolescent features
    select = ["E", "OB"]
    ```

    ----- stderr -----
    "#
    );
}

#[test]
fn check_select_json() {
    assert_cmd_snapshot!(
        Command::new(get_cargo_bin(BIN_NAME)).arg("config").arg("check.select").arg("--output-format").arg("json"), @r##"
    success: true
    exit_code: 0
    ----- stdout -----
    {
      "doc": "A list of rule codes or prefixes to enable. Prefixes can specify exact\nrules (like `T003` or `superfluous-implicit-none`), entire categories\n(like `T` or `typing`), or anything in between.\n\nWhen breaking ties between enabled and disabled rules (via `select` and\n`ignore`, respectively), more specific prefixes override less\nspecific prefixes.",
      "default": "[\"E\", \"F\", \"S\", \"T\", \"OB\", \"P\", \"M\", \"IO\", \"R\", \"B\"]",
      "value_type": "list[RuleSelector]",
      "scope": null,
      "example": "# Only check errors and obsolescent features\nselect = [\"E\", \"OB\"]",
      "deprecated": null
    }

    ----- stderr -----
    "##
    );
}
