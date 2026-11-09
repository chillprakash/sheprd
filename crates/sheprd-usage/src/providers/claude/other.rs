//! A second Claude account in a status bar slot of its own: the first
//! account its `accounts_file` lists that is not the active one, so a person
//! with two Claude Code profiles, such as work and personal, sees both at a
//! glance. The Claude provider keeps reporting the signed-in account and
//! still lists every account in its panel.

use super::listed;
use crate::text::Text;
use crate::{
    Result,
    model::{Account, Provider, Report},
    probe::Probe,
    service::{Meta, Service, Setting},
};

pub(crate) struct ClaudeOther;

static META: Meta = Meta::new("claudeother", "Claude (other account)")
    .icon("icons/agent-claude.svg")
    .dashboard("https://claude.ai/settings/usage")
    .status_page("https://status.anthropic.com")
    .settings(&[Setting::new(
        "accounts_file",
        &[],
        "usage-claudeother-setting-accounts-file",
    )]);

impl Service for ClaudeOther {
    fn meta(&self) -> &'static Meta {
        &META
    }

    fn fetch(&self, probe: &mut Probe) -> Option<Result<Report>> {
        let other = listed::accounts(probe)
            .others
            .into_iter()
            .find(|other| !other.windows.is_empty())?;
        let mut report = Report::new(
            Provider(&ClaudeOther),
            Account {
                email: Some(Text::service(&other.email)),
                plan: other.alias.as_deref().map(Text::service),
            },
            other.windows.clone(),
        );
        if other.stale {
            report = report.with_detail(vec![other]);
        }
        Some(Ok(report))
    }
}
