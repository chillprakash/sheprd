use super::*;
use crate::{probe::Shell, providers::claude::cswap::Other, settings::ProviderSettings};

const FILE: &str = r#"{"schemaVersion":1,"accounts":[
  {"email":"home@example.com","active":true,"usageStatus":"ok",
   "usage":{"fiveHour":{"pct":12.0,"resetsAt":1794240600},"sevenDay":{"pct":30.0}}},
  {"email":"work@example.com","alias":"WORK","usageStatus":"ok",
   "usage":{"fiveHour":{"pct":40.0},"sevenDay":{"pct":11.0}}}]}"#;

/// A host reached over a shell, as over SSH, with no Claude Code sign-in it
/// can read: not a Mac, so no keychain, and no `.credentials.json`.
fn host(file: Option<&str>) -> (tempfile::TempDir, Shell) {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let bin = home.join(".local/bin");
    std::fs::create_dir_all(&bin).unwrap();
    crate::test_executable::write(bin.join("uname"), "#!/bin/sh\necho Linux\n", 0o755).unwrap();
    if let Some(file) = file {
        std::fs::create_dir_all(home.join(".cache")).unwrap();
        std::fs::write(home.join(".cache/claude-accounts.json"), file).unwrap();
    }
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg("-s")
        .env_clear()
        .env("HOME", &home)
        .env("PATH", "/usr/bin:/bin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    (root, Shell::start(command).unwrap())
}

fn fetch(shell: Shell, settings: Option<&ProviderSettings>) -> Option<crate::Result<Report>> {
    fetch_as("claude", shell, settings)
}

fn fetch_as(
    id: &str,
    shell: Shell,
    settings: Option<&ProviderSettings>,
) -> Option<crate::Result<Report>> {
    let mut exec = Exec::Remote(shell);
    let mut jar = CookieJar::default();
    let claude = provider(id);
    let mut probe = Probe::new(&mut exec, claude, settings, &mut jar, Consent::Quiet);
    claude.service().fetch(&mut probe)
}

#[test]
fn a_listed_file_reports_accounts_without_a_readable_sign_in() {
    let (_root, shell) = host(Some(FILE));
    let settings =
        ProviderSettings::default().with("accounts_file", "~/.cache/claude-accounts.json");
    let report = fetch(shell, Some(&settings)).unwrap().unwrap();
    assert_eq!(
        report.account.email.as_ref().map(|email| email.as_str()),
        Some("home@example.com"),
        "the file's active account is the one reported"
    );
    assert_eq!(
        report
            .windows
            .iter()
            .map(|window| (window.kind.clone(), window.percent()))
            .collect::<Vec<_>>(),
        [(Kind::Session, 12), (Kind::Weekly, 30)]
    );
    let others = report.detail::<Vec<Other>>().unwrap();
    assert_eq!(
        others.iter().map(Other::name).collect::<Vec<_>>(),
        ["WORK"],
        "the rest are listed beneath it"
    );
}

#[test]
fn nothing_is_reported_without_the_setting_or_the_file() {
    let (_root, shell) = host(Some(FILE));
    assert!(
        fetch(shell, None).is_none(),
        "the file is read only when set"
    );

    let (_root, shell) = host(None);
    let settings =
        ProviderSettings::default().with("accounts_file", "~/.cache/claude-accounts.json");
    assert!(
        fetch(shell, Some(&settings)).is_none(),
        "a missing file lists nothing"
    );
}

#[test]
fn the_other_account_gets_a_slot_of_its_own() {
    let (_root, shell) = host(Some(FILE));
    let settings =
        ProviderSettings::default().with("accounts_file", "~/.cache/claude-accounts.json");
    let report = fetch_as("claudeother", shell, Some(&settings))
        .unwrap()
        .unwrap();
    assert_eq!(
        report.account.email.as_ref().map(|email| email.as_str()),
        Some("work@example.com")
    );
    assert_eq!(
        report.account.plan.as_ref().map(|label| label.as_str()),
        Some("WORK"),
        "the alias labels the slot"
    );
    assert_eq!(report.tightest().map(|window| window.percent()), Some(40));

    let only_active = r#"{"schemaVersion":1,"accounts":[
      {"email":"home@example.com","active":true,"usageStatus":"ok","usage":{"fiveHour":{"pct":1.0}}}]}"#;
    let (_root, shell) = host(Some(only_active));
    assert!(
        fetch_as("claudeother", shell, Some(&settings)).is_none(),
        "no second account, no slot"
    );
}
