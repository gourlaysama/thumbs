use assert_cmd::cargo::*;
use predicates::prelude::*;

#[test]
fn delete_no_arg() {
    let mut cmd = cargo_bin_cmd!("thumbs");

    cmd.arg("delete").assert().stderr(predicate::str::contains(
        "error: the following required arguments were not provided:",
    ));
}
