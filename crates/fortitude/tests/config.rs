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
    rules (like `S201` or `superfluous-implicit-none`), entire categories
    (like `C` or `correctness`), or anything in between.

    By default, a curated set of rules across all categories is enabled; see
    the documentation for details.

    When breaking ties between enabled and disabled rules (via `select` and
    `ignore`, respectively), more specific prefixes override less
    specific prefixes.

    Default value: []
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
      "doc": "A list of rule codes or prefixes to enable. Prefixes can specify exact\nrules (like `S201` or `superfluous-implicit-none`), entire categories\n(like `C` or `correctness`), or anything in between.\n\nBy default, a curated set of rules across all categories is enabled; see\nthe documentation for details.\n\nWhen breaking ties between enabled and disabled rules (via `select` and\n`ignore`, respectively), more specific prefixes override less\nspecific prefixes.",
      "default": "[]",
      "value_type": "list[RuleSelector]",
      "scope": null,
      "example": "# Only check errors and obsolescent features\nselect = [\"E\", \"OB\"]",
      "deprecated": null
    }

    ----- stderr -----
    "##
    );
}
