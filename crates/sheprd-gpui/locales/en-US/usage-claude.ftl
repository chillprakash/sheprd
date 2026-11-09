# Claude's plan usage, in its usage panel.

## Setup: how to find each value the provider reads. Setting names,
## environment variables, URLs, cookie and header names, and the
## service's own menu paths stay as written.

usage-claude-setting-accounts-file = Optional. A JSON file on the host, in the shape of `cswap list --json`, listing more Claude accounts to show beneath the signed-in one, e.g. "~/.cache/claude-accounts.json". A script that sees each account's limits, such as a Claude Code status line, can keep it, for accounts in other config directories or where the keychain cannot be read over SSH. Rows may add "updatedAt" (Unix seconds); one older than 15 minutes shows as last known.

## The report: this week's use by surface, and extra usage beyond the
## plan. Plan, model, and surface names (Claude Code, Chats) stay as
## Claude sends them.

usage-claude-week-by-surface = This week by surface
# Spending beyond the plan's limits, which the account may turn on.
usage-claude-extra-usage = Extra usage
usage-claude-this-month = This month
# Extra usage spent this month: $used of the monthly $limit, each an amount
# with its currency, such as 12.34 EUR.
usage-claude-spent-of = { $used } of { $limit }
# Extra usage turned on, with nothing spent reported.
usage-claude-on = On
usage-claude-status = Status
# Extra usage turned off.
usage-claude-off = Off
