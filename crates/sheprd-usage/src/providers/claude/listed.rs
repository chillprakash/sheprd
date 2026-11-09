//! Claude accounts listed in a file the user keeps, named by the
//! `accounts_file` setting, in the shape of `cswap list --json`. A script
//! that sees each account's limits, such as a Claude Code status line given
//! each session's `rate_limits`, can write it, so accounts in other config
//! directories show without reading their sign-ins. That also works where
//! the keychain cannot be read, such as a Mac probed over SSH. The file holds
//! no secrets and is read on the probed host.

use super::cswap::{self, Accounts};
use crate::probe::{HostPath, Probe};

/// The accounts the file lists; none when the setting is unset, or the file
/// is missing or not in a shape this reads.
pub(super) fn accounts(probe: &mut Probe) -> Accounts {
    probe
        .text_setting("accounts_file")
        .filter(|path| !path.is_empty())
        .and_then(|path| probe.read(&host_path(&path)))
        .and_then(|body| cswap::parse(&body).ok())
        .unwrap_or_default()
}

/// `~/` is the probed host's home; anything else is taken as absolute.
fn host_path(path: &str) -> HostPath {
    match path.strip_prefix("~/") {
        Some(rest) => HostPath::home(rest),
        None => HostPath::absolute(path),
    }
}

impl Accounts {
    /// These accounts, then `more`'s, each email once. The first active
    /// account stays active; a later one joins the others.
    pub(super) fn merge(self, more: Accounts) -> Accounts {
        let Accounts { active, others } = self;
        let (active, demoted) = match (active, more.active) {
            (Some(active), later) => (Some(active), later),
            (None, later) => (later, None),
        };
        let mut merged = Accounts {
            active,
            others: Vec::new(),
        };
        for other in others.into_iter().chain(demoted).chain(more.others) {
            let seen = merged
                .active
                .iter()
                .chain(&merged.others)
                .any(|kept| kept.email.eq_ignore_ascii_case(&other.email));
            if !seen {
                merged.others.push(other);
            }
        }
        merged
    }
}

#[cfg(test)]
mod tests;
