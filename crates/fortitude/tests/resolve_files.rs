#![cfg(not(target_family = "wasm"))]

use std::path::Path;
use std::process::Command;
use std::str;

use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
const BIN_NAME: &str = "fortitude";

#[cfg(not(target_os = "windows"))]
const TEST_FILTERS: &[(&str, &str)] = &[(".*/resources/test/fixtures/", "[BASEPATH]/")];
#[cfg(target_os = "windows")]
const TEST_FILTERS: &[(&str, &str)] = &[
    (r".*\\resources\\test\\fixtures\\", "[BASEPATH]\\"),
    (r"\\", "/"),
];

#[test]
fn check_project_include_defaults() {
    // Defaults to checking the current working directory
    //
    // The test directory includes:
    //  - A fortitude.toml which specifies an include
    //  - A nested fortitude.toml which has a Fortitude section
    //
    // The nested project should all be checked instead of respecting the parent includes

    insta::with_settings!({
        filters => TEST_FILTERS.to_vec()
    }, {
        assert_cmd_snapshot!(Command::new(get_cargo_bin(BIN_NAME))
        .args(["check", "--show-files"]).current_dir(Path::new("./resources/test/fixtures/include-test")), @r"
        success: true
        exit_code: 0
        ----- stdout -----
        [BASEPATH]/include-test/a.f90
        [BASEPATH]/include-test/nested-project/e.f90
        [BASEPATH]/include-test/subdirectory/c.f90

        ----- stderr -----
        ");
    });
}

#[test]
fn check_project_respects_direct_paths() {
    // Given a direct path not included in the project `includes`, it should be checked

    insta::with_settings!({
        filters => TEST_FILTERS.to_vec()
    }, {
        assert_cmd_snapshot!(Command::new(get_cargo_bin(BIN_NAME))
        .args(["check", "--show-files", "b.f90"]).current_dir(Path::new("./resources/test/fixtures/include-test")), @r"
        success: true
        exit_code: 0
        ----- stdout -----
        [BASEPATH]/include-test/b.f90

        ----- stderr -----
        ");
    });
}

#[test]
fn check_project_respects_subdirectory_includes() {
    // Given a direct path to a subdirectory, the include should be respected

    insta::with_settings!({
        filters => TEST_FILTERS.to_vec()
    }, {
        assert_cmd_snapshot!(Command::new(get_cargo_bin(BIN_NAME))
        .args(["check", "--show-files", "subdirectory"]).current_dir(Path::new("./resources/test/fixtures/include-test")), @r"
        success: true
        exit_code: 0
        ----- stdout -----
        [BASEPATH]/include-test/subdirectory/c.f90

        ----- stderr -----
        ");
    });
}

#[test]
fn check_project_from_project_subdirectory_respects_includes() {
    // Run from a project subdirectory, the include specified in the parent directory should be respected

    insta::with_settings!({
        filters => TEST_FILTERS.to_vec()
    }, {
        assert_cmd_snapshot!(Command::new(get_cargo_bin(BIN_NAME))
        .args(["check", "--show-files"]).current_dir(Path::new("./resources/test/fixtures/include-test/subdirectory")), @r"
        success: true
        exit_code: 0
        ----- stdout -----
        [BASEPATH]/include-test/subdirectory/c.f90

        ----- stderr -----
        ");
    });
}
