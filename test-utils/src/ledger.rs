//! The engine half of `docs/contracts/runtime-adjustments.tsv`, the ledger of every adjustment that
//! reaches one title only. Each per-title table calls [`assert_ledger_lists`] from its own crate's
//! tests with the keys it compiled — the table itself, not a transcription — so a row added to the
//! engine without a ledger row (or the reverse) fails `cargo test`. The other half, ledger →
//! `compat.json`'s `runtimeAdjustments`, is `scripts/player-data.mjs`.

use alloc::{collections::BTreeSet, format, string::String, vec::Vec};

/// The ledger as committed, `#` comment lines included.
pub const LEDGER: &str = include_str!("../../docs/contracts/runtime-adjustments.tsv");

const COLUMNS: &str = "device\tkey\tpackage\tplatform\ttitle\tkind\tchanges\tscope\tclass\tbasis";

/// Every engine table the ledger covers. A table added here needs its own `assert_ledger_lists` call.
const DEVICES: [&str; 11] = [
    "binary-patches",
    "class-patches",
    "owner-sound-off-settings",
    "owner-sound-off-bytes",
    "owner-sound-off-xor",
    "orphan-entries",
    "display-overrides",
    "device-bound-saves",
    "owner-progress-saves",
    "fake-servers",
    "ktf-local-servers",
];

/// The ledger's rows, one per (engine key, package), as their tab-separated cells.
fn rows() -> Vec<Vec<&'static str>> {
    let mut lines = LEDGER.lines().filter(|line| !line.starts_with('#') && !line.is_empty());
    assert_eq!(lines.next(), Some(COLUMNS), "the ledger's header row");

    let rows = lines.map(|line| line.split('\t').collect::<Vec<_>>()).collect::<Vec<_>>();
    for row in &rows {
        assert_eq!(
            row.len(),
            COLUMNS.split('\t').count(),
            "a ledger row with the wrong number of tab-separated cells: {row:?}"
        );
    }
    rows
}

/// The key of each ledger row for `device` — a key shared by two packages comes twice.
pub fn ledger_keys(device: &str) -> Vec<&'static str> {
    rows().iter().filter(|row| row[0] == device).map(|row| row[1]).collect()
}

/// An MD5 as the ledger spells it.
pub fn hex(md5: [u8; 16]) -> String {
    md5.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Fails unless the ledger's rows for `device` name exactly the keys of the engine table, and each
/// of those rows says where it came from.
pub fn assert_ledger_lists(device: &str, engine_keys: impl IntoIterator<Item = String>) {
    const WHERE: &str = "docs/contracts/runtime-adjustments.tsv (CLAUDE.md «게임 파일 무수정 원칙»)";
    assert!(DEVICES.contains(&device), "{device} is not a table this guard knows");

    let engine = engine_keys.into_iter().collect::<BTreeSet<_>>();
    let rows = rows().into_iter().filter(|row| row[0] == device).collect::<Vec<_>>();
    let listed = rows.iter().map(|row| String::from(row[1])).collect::<BTreeSet<_>>();

    let unlisted = engine.difference(&listed).collect::<Vec<_>>();
    let stale = listed.difference(&engine).collect::<Vec<_>>();
    assert!(
        unlisted.is_empty() && stale.is_empty(),
        "{device}: engine rows the ledger does not list {unlisted:?}, ledger rows the engine does not have {stale:?} — {WHERE}"
    );
    for row in rows {
        assert!(
            !row[9].trim().is_empty(),
            "{device} {}: a ledger row must say where it came from (`basis`) — {WHERE}",
            row[1]
        );
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use super::{DEVICES, rows};

    /// A ledger table no engine test reads would mark titles with nothing behind the mark.
    #[test]
    fn the_ledger_names_only_tables_the_guard_reads() {
        assert_eq!(rows().iter().map(|row| row[0]).collect::<BTreeSet<_>>(), BTreeSet::from(DEVICES));
    }
}
