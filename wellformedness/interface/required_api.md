# Required public API (interoperability surface)

```rust
pub struct WfError { pub topic: String, pub message: String }
impl WfError { pub fn new(topic: impl Into<String>, message: impl Into<String>) -> Self }
pub type WfReport = Vec<WfError>;

/// Run every check in the oracle's report order.
pub fn check_theory(thy: &Theory) -> WfReport;
/// Set of distinct topics in a report.
pub fn topics(report: &WfReport) -> std::collections::BTreeSet<String>;
/// A topic header formatted exactly as the oracle renders it.
pub fn underline_topic(title: &str) -> String;
/// Render a full report as the oracle's WARNING text (byte-identical),
/// or the oracle's success line for an empty report.
pub fn render_report(report: &WfReport) -> String;

// Secondary entry points (same behavior contracts, discoverable via oracle):
pub fn check_if_lemmas_in_theory(lemma_names: &[String], thy: &Theory) -> WfReport;
pub fn fact_lhs_occur_no_rhs(thy: &Theory) -> WfReport;
pub fn public_names_report(thy: &Theory) -> WfReport;
pub fn public_names_report_from_pairs(pairs: Vec<(String, String)>) -> WfReport;
pub fn insert_wf_before(report: &mut Vec<WfError>, errors: Vec<WfError>, anchors: &[&str]);
pub fn after_public_names_topics() -> Vec<&'static str>;
```
`Theory` and its components: see ast_types.rs.
