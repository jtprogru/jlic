//! Integration tests for the binary.
//!
//! Name and email are always supplied through the environment so results do
//! not depend on the git config of the machine running the suite.

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn jlic(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("jlic").unwrap();
    cmd.current_dir(dir.path())
        .env("JLIC_NAME", "Mikhail Savin")
        .env("JLIC_EMAIL", "jtprogru@gmail.com");
    cmd
}

/// A command with no identity available anywhere: no environment, and git
/// pointed at config files that do not exist (which git reads as empty).
fn anonymous(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("jlic").unwrap();
    cmd.current_dir(dir.path())
        .env_remove("JLIC_NAME")
        .env_remove("JLIC_EMAIL")
        .env("HOME", dir.path())
        .env("GIT_CONFIG_GLOBAL", dir.path().join("no-such-gitconfig"))
        .env("GIT_CONFIG_SYSTEM", dir.path().join("no-such-gitconfig"));
    cmd
}

#[test]
fn bare_invocation_writes_mit_license() {
    let dir = TempDir::new().unwrap();
    jlic(&dir).assert().success();

    let text = std::fs::read_to_string(dir.path().join("LICENSE")).unwrap();
    assert!(text.starts_with("MIT License"));
    assert!(text.contains("Copyright (c) "));
    assert!(text.contains("Mikhail Savin <jtprogru@gmail.com>"));
}

#[test]
fn writes_requested_license_to_requested_path() {
    let dir = TempDir::new().unwrap();
    jlic(&dir)
        .args([
            "new",
            "apache-2.0",
            "-o",
            "docs/LICENSE.txt",
            "--year",
            "2020-2026",
        ])
        .assert()
        .success();

    let text = std::fs::read_to_string(dir.path().join("docs/LICENSE.txt")).unwrap();
    assert!(text.contains("Apache License"));
    assert!(text.contains("Copyright 2020-2026 Mikhail Savin <jtprogru@gmail.com>"));
}

#[test]
fn refuses_to_overwrite_without_force() {
    let dir = TempDir::new().unwrap();
    jlic(&dir).assert().success();

    jlic(&dir)
        .arg("wtfpl")
        .assert()
        .failure()
        .stderr(contains("already exists"));

    jlic(&dir).args(["wtfpl", "--force"]).assert().success();
    let text = std::fs::read_to_string(dir.path().join("LICENSE")).unwrap();
    assert!(text.contains("DO WHAT THE FUCK YOU WANT TO"));
    assert!(text.contains("Copyright (C) "));
}

#[test]
fn no_email_flag_drops_email_from_copyright() {
    let dir = TempDir::new().unwrap();
    let out = jlic(&dir)
        .args(["mit", "--stdout", "--no-email"])
        .assert()
        .success();

    let text = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(text.contains("Mikhail Savin"));
    assert!(!text.contains("jtprogru@gmail.com"));
}

#[test]
fn missing_name_is_a_clear_error() {
    let dir = TempDir::new().unwrap();
    anonymous(&dir)
        .args(["mit", "--stdout"])
        .assert()
        .failure()
        .stderr(contains("cannot determine the copyright holder"));
}

#[test]
fn fixed_text_license_renders_without_any_identity() {
    let dir = TempDir::new().unwrap();
    anonymous(&dir)
        .args(["gpl-3.0-or-later", "--stdout"])
        .assert()
        .success()
        .stdout(contains("GNU GENERAL PUBLIC LICENSE"));
}

#[test]
fn warns_when_identity_cannot_be_used() {
    let dir = TempDir::new().unwrap();
    jlic(&dir)
        .args(["mpl-2.0", "--stdout", "--name", "Someone"])
        .assert()
        .success()
        .stderr(contains("has no copyright line"));
}

#[test]
fn unknown_license_suggests_alternative() {
    let dir = TempDir::new().unwrap();
    jlic(&dir)
        .arg("apach")
        .assert()
        .failure()
        .stderr(contains("unknown license"))
        .stderr(contains("Apache-2.0"));
}

#[test]
fn list_shows_every_license() {
    let dir = TempDir::new().unwrap();
    let mut assertion = jlic(&dir).arg("list").assert().success();
    for id in [
        "MIT",
        "Apache-2.0",
        "GPL-3.0-or-later",
        "BSD-3-Clause",
        "MPL-2.0",
        "ISC",
        "WTFPL",
    ] {
        assertion = assertion.stdout(contains(id));
    }
}

#[test]
fn list_json_is_machine_readable() {
    let dir = TempDir::new().unwrap();
    let out = jlic(&dir).args(["list", "--json"]).assert().success();
    let text = String::from_utf8(out.get_output().stdout.clone()).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let items = parsed.as_array().unwrap();
    assert_eq!(items.len(), 7);
    assert_eq!(items[0]["id"], "MIT");
    assert_eq!(items[0]["placeholders"]["year"], true);
}

#[test]
fn show_prints_raw_template() {
    let dir = TempDir::new().unwrap();
    jlic(&dir)
        .args(["show", "mit"])
        .assert()
        .success()
        .stdout(contains("{{year}}"))
        .stdout(contains("{{holder}}"));
}

#[test]
fn notice_carries_spdx_identifier() {
    let dir = TempDir::new().unwrap();
    jlic(&dir)
        .args(["notice", "gpl-3.0-or-later", "--year", "2026"])
        .assert()
        .success()
        .stdout(contains(
            "Copyright (C) 2026 Mikhail Savin <jtprogru@gmail.com>",
        ))
        .stdout(contains("SPDX-License-Identifier: GPL-3.0-or-later"));
}

#[test]
fn completions_and_man_are_generated() {
    let dir = TempDir::new().unwrap();
    jlic(&dir)
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(contains("_jlic"));

    jlic(&dir)
        .arg("man")
        .assert()
        .success()
        .stdout(contains("JLIC"));
}
