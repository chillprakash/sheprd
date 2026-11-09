#![allow(clippy::unwrap_used)]

use super::*;
use crate::providers::claude::cswap::{Other, parse};
use std::time::SystemTime;

fn names(accounts: &Accounts) -> (Option<&str>, Vec<&str>) {
    (
        accounts.active.as_ref().map(Other::name),
        accounts.others.iter().map(Other::name).collect(),
    )
}

fn list(rows: &str) -> Accounts {
    parse(&format!(r#"{{"schemaVersion":1,"accounts":[{rows}]}}"#)).unwrap()
}

#[test]
fn merge_keeps_the_first_active_account_and_each_email_once() {
    let cswap = list(
        r#"{"email":"me@example.com","active":true,"usageStatus":"ok","usage":{"fiveHour":{"pct":5.0}}},
           {"email":"work@example.com","usageStatus":"ok","usage":null}"#,
    );
    let file = list(
        r#"{"email":"home@example.com","active":true,"usageStatus":"ok","usage":{"fiveHour":{"pct":9.0}}},
           {"email":"WORK@example.com","usageStatus":"ok","usage":{"sevenDay":{"pct":1.0}}},
           {"email":"me@example.com","usageStatus":"ok","usage":null}"#,
    );
    let merged = cswap.merge(file);
    assert_eq!(
        names(&merged),
        (
            Some("me@example.com"),
            vec!["work@example.com", "home@example.com"]
        ),
        "the file's active account joins the others; repeated emails are dropped"
    );
}

#[test]
fn the_file_supplies_the_active_account_when_cswap_has_none() {
    let file = list(
        r#"{"email":"home@example.com","active":true,"usageStatus":"ok","usage":{"fiveHour":{"pct":9.0}}},
           {"email":"work@example.com","usageStatus":"ok","usage":{"fiveHour":{"pct":3.0}}}"#,
    );
    let merged = Accounts::default().merge(file);
    assert_eq!(
        names(&merged),
        (Some("home@example.com"), vec!["work@example.com"])
    );
}

#[test]
fn an_old_updated_at_shows_as_last_known() {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let accounts = list(&format!(
        r#"{{"email":"fresh@example.com","usageStatus":"ok","updatedAt":{fresh},"usage":{{"fiveHour":{{"pct":1.0}}}}}},
           {{"email":"old@example.com","usageStatus":"ok","updatedAt":{old},"usage":{{"fiveHour":{{"pct":2.0}}}}}},
           {{"email":"plain@example.com","usageStatus":"ok","usage":{{"fiveHour":{{"pct":3.0}}}}}}"#,
        fresh = now - 60,
        old = now - 3600,
    ));
    let stale: Vec<_> = accounts
        .others
        .iter()
        .map(|other| (other.email.as_str(), other.stale, other.note()))
        .collect();
    assert_eq!(
        stale,
        [
            ("fresh@example.com", false, None),
            ("old@example.com", true, Some("last known")),
            ("plain@example.com", false, None),
        ]
    );
}

#[test]
fn paths_are_home_relative_or_absolute() {
    assert_eq!(
        host_path("~/.cache/a.json"),
        HostPath::home(".cache/a.json")
    );
    assert_eq!(host_path("/var/a.json"), HostPath::absolute("/var/a.json"));
}
