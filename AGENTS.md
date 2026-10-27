# Native port workspace

These rules apply to every task in this workspace. The user's requested task
selects the workflow; this file does not start or resume a goal.

- Use Rust 1.90.0 and Cargo.lock. Do not launch desktop graphics on this
  headless server; compile desktop code without opening a window.
- Prefer --release for Cargo builds, runs, tests and Clippy. Keep incremental
  compilation enabled and debug symbols disabled.
- Preserve reference/, review/toolkit-before/, original ledger identities,
  deployment backups and unrelated existing changes. Keep QA records separate
  from implementation accounting and preserve superseded records.
- Use the shared Game APIs for desktop/browser rendering, audio, controls and
  original battery saves. Keep identity, dimensions, cadence and defaults in
  config.rs where present. Keep game rules shared across frontends.
- Toolkit: "/root/romatlas". For edits there, follow that repository's applicable
  instructions. Leave shared-library edits uncommitted and log their changes
  and check results in library-changes.md; preserve pre-existing work.
- Invoking goal.md authorizes its fixed five-stage pipeline and per-area contract subagents.
  Preparation alone launches no agents. No check-in or publication is authorized.
