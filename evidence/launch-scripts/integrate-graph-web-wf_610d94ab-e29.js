export const meta = {
  name: 'integrate-graph-web',
  description: 'Dirty-room integration: swap tamarin-server onto the audited graph-clean and web-clean crates',
  phases: [{ title: 'Integrate' }],
}
phase('Integrate')
const out = await agent(`You are the DIRTY-ROOM INTEGRATOR in a clean-room relicensing process (read
/home/kamilner/tamarin-cleanroom/PROTOCOL.md). You may read everything, but you must NOT transplant
logic from the files being replaced — you write ADAPTERS (type conversion, data plumbing) that route
existing entry points through the audited clean-room crates. Where the clean crate lacks a behavior,
you leave the old path in place and report the gap; you never patch clean code with old-code logic.

REPO: /home/kamilner/tamarin-rs (workspace). CLEAN CRATES:
/home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean (model.rs, term.rs, abbrev.rs incl.
round-2 abbrev::select, dot.rs — 12022/12022 byte-exact DOT round-trip, selection policy validated)
/home/kamilner/tamarin-cleanroom/weblayer/workspace/web-clean (route, envelope, escape, page+
shell_template, proofscript, forms, intdot, errors+notfound_template, text — 15178/15178 captured
HTML bodies byte-exact, 2450 JSON envelope bodies byte-exact).

PRECEDENT: the wellformedness cluster was integrated by vendoring clean sources as an in-crate module
(crates/tamarin-parser/src/wf/) with mechanical path fixes plus a small workspace-authored adapter
file (wf/order.rs). Follow the same pattern.

TASKS (sequential, keep the build green after each):
1. GRAPH: vendor graph-clean sources as crates/tamarin-server/src/graph_clean/ (mechanical path
   fixes only; no GPL headers on these files — they are the clean deliverable). Rewire the DOT
   pipeline entry points in handlers/dot.rs (system_to_dot at :142, system_to_dot_with :150,
   render_svg_or_dot_with :435) to: existing repr/simplify stages (KEEP graph/repr.rs and
   graph/simplify.rs and graph/options.rs — ported, still headered, out of scope this round) ->
   ADAPTER converting the resulting graph representation into graph_clean's model -> graph_clean
   abbreviation selection/naming -> graph_clean DOT serialization. Then DELETE
   crates/tamarin-server/src/graph/abbreviation.rs and all now-dead serialization/abbreviation code
   in handlers/dot.rs (ideally the whole file becomes a thin adapter; whatever ported code must
   remain, keep its GPL header and say why it remains).
2. WEB: vendor web-clean as crates/tamarin-server/src/web_clean/. Rewire
   crates/tamarin-server/src/handlers/theory_html.rs callers to render via web_clean templates
   (shell/page, intdot, forms, 404, envelopes), passing prover-generated content (proof scripts,
   theory pretty-prints from tamarin-theory) as opaque pane/body strings — web_clean models panes
   as opaque, so the adapter supplies them. DELETE theory_html.rs when its render logic is fully
   replaced (move any pure data-plumbing callers need into a small adapter module). If some page
   family cannot be routed through web_clean without behavior change, keep that path, keep its
   header, and report precisely what blocks it.
3. VALIDATE after each task and at the end: cargo build --workspace (0 errors), cargo test -p
   tamarin-server (all suites — these include captured-HS-response parity fixtures and MUST stay
   green), cargo test -p tamarin-parser, and the graph fixture tests. If a parity test fails after
   your rewiring, the adapter is wrong — fix the ADAPTER, never the clean code, and never by
   restoring old logic outside the kept files.
4. HEADERS: deleted files' headers go with them. Files you author carry NO license header. Do not
   touch headers of files you keep.
Write /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md (what was replaced, what remains ported
and why, LOC deleted vs added, test results) and return a 12-line plain-text summary. No structured output.`,
  { label: 'integrate:graph+web', phase: 'Integrate', model: 'opus' })
return { out }