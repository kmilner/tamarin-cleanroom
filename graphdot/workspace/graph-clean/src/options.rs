//! Graph simplification / abbreviation options (BEHAVIOR.md §5, §7).
//!
//! The web UI exposes four "Graph simplification" levels (`lvl0`..`lvl3`
//! toggles: *off*, *L1*, *L2*, *L3*) plus an abbreviation toggle. These are not
//! separate endpoints — the client JS stores them in cookies and appends them as
//! **query parameters** to the graph URL. The observed parameter set (mined from
//! the UI JS and confirmed by live probes; see QUERIES.log) is:
//!
//! | param            | when the UI sends it            | server-side effect (observed)             |
//! |------------------|---------------------------------|-------------------------------------------|
//! | `simplification=N` | always (N = level, default 2) | selects the level; N alone changed **no** probed graph (L1≡L2≡L3 byte-identical) |
//! | `uncompact`      | only at level 0                 | disables node COMPACTION                  |
//! | `uncompress`     | only at level 0                 | disables system COMPRESSION               |
//! | `unabbreviate`   | when the abbreviate cookie is off | disables the abbreviation pass          |
//! | `no-auto-sources`| when auto-sources cookie is off | disables auto-source-lemma precomputation |
//! | `clustering=true`| when clustering cookie is on    | forces role clustering                    |
//!
//! Observed transforms (live diffs, NAXOS/NSLPK3 case graphs):
//!   * **compression** (on unless `uncompress`): Fresh rules (`Fr( ~x )` sources)
//!     and single intruder-derivation rules are hidden / collapsed — `!KU` facts
//!     become gray ellipses and Fresh sources disappear; turning it OFF re-expands
//!     them into full `record` rule nodes (node/edge counts grow markedly).
//!   * **compaction** (on unless `uncompact`): a lone intruder `isend`/`coerce`
//!     rule is drawn as a single ellipse (e.g. `#vf.7 : isend`); OFF draws it as a
//!     full `{!KU(m)}|{#vf : isend[K(m)]}|{In(m)}` record.
//!   * **abbreviation** (on unless `unabbreviate`): complex sub-terms are replaced
//!     by legend names and a `{ rank="sink"; … }` legend block is emitted; OFF
//!     inlines every term in full and omits the legend block entirely.
//!
//! The default (no query params, as the corpus was crawled) equals level ≥1:
//! compressed + compacted + abbreviated. The distinction between L1/L2/L3 was not
//! reproduced on any probed graph and is a documented GAP — as is the *content* of
//! compression/compaction, which needs the GPL solver (this crate models the
//! flags and the abbreviation on/off transform, taking the node set as input).

/// The observed graph-rendering options. `abbreviate` is the one transform this
/// crate applies directly (drop the legend + inline terms when false); the
/// compress/compact flags are recorded and documented — their *content* is a
/// solver-side GAP, so a caller supplies the already-(un)compressed node set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Graph simplification level 0..=3 (UI default 2).
    pub level: u8,
    /// Run the abbreviation pass (emit a legend). `false` ⇒ `unabbreviate`.
    pub abbreviate: bool,
    /// Apply node compaction. `false` ⇒ `uncompact` (sent by the UI at level 0).
    pub compact: bool,
    /// Apply system compression. `false` ⇒ `uncompress` (sent by the UI at level 0).
    pub compress: bool,
    /// Force role clustering (`clustering=true`).
    pub clustering: bool,
}

impl Default for Options {
    /// The corpus/default rendering: level 2, abbreviated, compacted, compressed.
    fn default() -> Self {
        Options { level: 2, abbreviate: true, compact: true, compress: true, clustering: false }
    }
}

impl Options {
    /// The UI's level-0 preset: compaction and compression OFF (the UI sends
    /// `uncompact` + `uncompress` + `simplification=0`).
    pub fn level0() -> Self {
        Options { level: 0, compact: false, compress: false, ..Options::default() }
    }

    /// Reconstruct the query string the UI appends for these options, in the
    /// UI's parameter order (`uncompact`, `uncompress` first — only at level 0 —
    /// then `unabbreviate`, then `simplification`, then `clustering`). Returns the
    /// part after `?` (empty if nothing would be appended). Byte-shaped to match
    /// the observed `?uncompact=&uncompress=&simplification=0` form.
    pub fn query_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.level == 0 {
            parts.push("uncompact=".into());
            parts.push("uncompress=".into());
        }
        if !self.abbreviate {
            parts.push("unabbreviate=".into());
        }
        parts.push(format!("simplification={}", self.level));
        if self.clustering {
            parts.push("clustering=true".into());
        }
        parts.join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_query_is_level_2() {
        assert_eq!(Options::default().query_string(), "simplification=2");
    }

    #[test]
    fn level0_sends_uncompact_uncompress() {
        // Matches the live URL that produced the expanded graph:
        //   ?uncompact=&uncompress=&simplification=0
        assert_eq!(Options::level0().query_string(), "uncompact=&uncompress=&simplification=0");
    }

    #[test]
    fn unabbreviate_param() {
        let o = Options { abbreviate: false, ..Options::default() };
        assert_eq!(o.query_string(), "unabbreviate=&simplification=2");
    }

    #[test]
    fn raw_all_off() {
        let o = Options { level: 0, abbreviate: false, compact: false, compress: false, clustering: false };
        assert_eq!(o.query_string(), "uncompact=&uncompress=&unabbreviate=&simplification=0");
    }
}
