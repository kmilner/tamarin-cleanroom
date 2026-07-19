export const meta = {
  name: 'pretty-r5-integration',
  description: 'Open-side pretty round-5 integration: re-sync, route restrictions + macros + whole-echo frame, full gates',
  phases: [{ title: 'Integrate', detail: 're-sync + remaining echo routes (Opus)' }],
}

const GATE = `You are the OPEN-SIDE integrator for the tamarin-rs sealed relicensing campaign, working in /home/kamilner/tamarin-rs. Read first: the pretty sections of /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md (endgame round: rule + lemma routes ADOPTED byte-green; restriction blocked on the single-formula model, macros blocked on sep-vs-vcat, frame blocked on unimplemented theory::render; the batch/web width gate via pretty_hpj::display_line_length), and PROTOCOL.md.
CONTEXT: sealed round 5 closed all three blockers — Restriction.expanded (opaque caller-supplied expanded formula; statement renders the macro-form field, comment the expanded field), macros_doc vcat always-break law, and theory::render (whole-echo frame assembly, 29/29 capture parity; plus render_predicates with the margin-0 splice law and TheoryItem extensions incl. Heuristic/Verbatim).
RULES: mechanical re-sync (crate::->super:: recipe; headerless; reverse-transform byte-identity); adapters translate values only (the expanded formula comes from the ported macro expansion — supply it as input, never move expansion logic); batch routes only (the web panes stay ported per the width/HtmlDoc gate). No git commits. PATH needs /home/linuxbrew/.linuxbrew/bin.
STEPS:
1. Re-sync round-5 sources into crates/tamarin-theory/src/pretty_clean/ (ast/lemma/macros/theory/lib changed). Extend pretty_clean_adapt.rs (Restriction.expanded from the ported expansion, TheoryItem inputs). Build + full workspace tests green.
2. ROUTE, each gated by JOBS=6 bash scripts/pretty_gate.sh expecting EXACTLY 403 MATCH / 16 DIFF (revert offender on ANY new DIFF, keep the rest, record witnesses): (a) restrictions; (b) the macros block; (c) predicates if the ported echo emits them separately; (d) OPTIONALLY the whole-echo frame via clean render_theory — only if byte-green; partial block-level routing is fine as the end state.
3. Full gates: cargo build --release; cargo test --workspace; pretty_gate final; JOBS=6 bash scripts/wf_gate.sh (419/0 vs scripts/wf_gate_round7.tsv); bash scripts/web_parity.sh; console_split_parity; scripts/gen_license_headers.py --check (0 stale, no clean file headered). Report header delta (expect 0 — the web panes still hold the ported files; that analysis is done, do not repeat it).
4. Append a dated pretty round-5 integration section to INTEGRATION_REPORT.md. Return a plain-text summary: surfaces routed, gate numbers, anything reverted with witnesses.`

const gate = await agent(GATE, { label: 'integrate:pretty-r5', phase: 'Integrate', model: 'opus' })
return { gate: (gate || '').slice(0, 4000) }