# Evidence archive — tamarin-rs clean-room relicensing campaign

Assembled 2026-07-17 from the orchestrating session's persistent stores. Every
agent in this campaign ran under a harness that logs each tool invocation
(file reads, shell commands, web access) to a per-agent JSONL transcript;
those transcripts are collected here verbatim. `SHA256SUMS` covers all files.

## Clean-room implementer transcripts (core evidence)

These agents were REQUIRED to work only from black-box oracles and captured
program output (see ../PROTOCOL.md). Their transcripts prove non-access.
Final audit (audit/cleanroom_audit_results.txt): 36 transcripts, 2837 tool
calls, 0 forbidden accesses, 0 web accesses (wave-4's six implementer
transcripts — including the crash-orphaned first B8 attempt and the two
post-resume verification re-runs — audited 2026-07-18). Audit-rule note (2026-07-18):
the checker now classifies reads of `.spthy` example theories via the raw
submodule path as sanctioned input (they always were under PROTOCOL.md; only
symlinked access was whitelisted before), and gained a stricter override that
flags direct `.hs`/`crates/` path use even alongside allowed substrings. The
full historical set re-validates at 0 violations under the stricter rule.
2026-07-18 (later): wf round-5 implementer (wf_b6a947c0-a82) audited — 153
tool calls, 0 forbidden accesses, 0 web accesses; graphdot round-9 implementer
(wf_6b16799d-1c9) audited — 156 tool calls, 0/0; producers round-1 implementer
(wf_f9696b2b-75e) audited — 72 tool calls, 0/0; graphdot round-10 implementer
(wf_58b2a176-d66) audited — 148 tool calls, 0/0; wf round-6 + pretty round-1
implementers (wf_cdfa493d-262) audited — 49 + 88 tool calls, 0/0;
producers round-2 implementer (wf_910e7687-413) audited — 103 tool calls, 0/0;
graphdot round-11 implementer (wf_6c1add5e-f10) audited — 149 tool calls, 0/0;
producers round-3 implementer (wf_e914d4ec-17f) audited — 131 tool calls, 0/0;
pretty round-2 implementer (wf_a5ef3578-648) audited — 105 tool calls, 0/0;
wf round-7 implementer (wf_bf6a23fd-437) audited — 77 tool calls, 0/0;
graphdot round-12 implementer (wf_8ba08a93-8eb) audited — 115 tool calls, 0/0;
pretty round-3 implementer (wf_1c3b9e54-e0c) audited — 143 tool calls, 0/0 (2 file_flags.tsv reads sanctioned by the dated rule refinement; all 49 audited transcripts re-validate at 0 under it);
pretty round-4 implementer (wf_e752c3f6-6f4) audited — 131 tool calls, 0/0;
graphdot round-13 implementer (wf_92d2ed53-daf) audited — 69 tool calls, 0/0;
pretty round-5 implementer (wf_a320cb04-549) audited — 94 tool calls, 0/0;
52 sealed transcripts total.

| Path | Agent | Role |
|------|-------|------|
| workflows/wf_d28aad0e-98c/agent-a169c9fd75138f333.jsonl | round 1 | wellformedness implementer |
| workflows/wf_d28aad0e-98c/agent-a29d812038b601a9c.jsonl | round 1 | graph/dot implementer |
| workflows/wf_d28aad0e-98c/agent-a9addc4f0d5b97ed0.jsonl | round 1 | web-layer implementer |
| workflows/wf_d28aad0e-98c/agent-a4b479b960c441512.jsonl | round 1 cont. | web-layer continuation |
| workflows/wf_6e112628-91d/agent-ae38332cc1eb5f790.jsonl | round 2 | wellformedness gap-closer |
| workflows/wf_db8cc474-1b6/agent-aa7e30d4b753e4fb7.jsonl | round 2 | graph abbreviation-selection miner |
| workflows/wf_db8cc474-1b6/agent-acd8fb9a8d13fc15f.jsonl | round 2 | web page-generality |
| workflows/wf_7f807521-670/agent-a27fa6413061022ae.jsonl | round 3 (unit C) | wellformedness completion |
| workflows/wf_7f807521-670/agent-ad2408a9cda7bbd15.jsonl | round 3 (unit D) | console/CLI implementer |
| workflows/wf_7f807521-670/agent-a2df8d6d0f01ac441.jsonl | round 3 (unit E) | macro-expansion implementer |
| workflows/wf_7f807521-670/agent-a842511ba769df7f3.jsonl | round 3 (unit F) | injective-facts implementer |
| workflows/wf_7f807521-670/agent-a4c19ed19405e64bb.jsonl | round 3 (unit G) | derivation-checks implementer |
| workflows/wf_7f807521-670/agent-a7d8ac2b487df1e33.jsonl | round 3 (unit A) | web interactive-UI state machine |
| workflows/wf_7f807521-670/agent-ac1c391bc2c48679f.jsonl | round 3 (unit B) | graph generation implementer |
| workflows/wf_bfd5a6a0-31b/agent-abad7225bcac84f4e.jsonl | round 3 cont. (unit F) | corpus-validation rebuild (36/36 agree) |
| workflows/wf_62e2c0ef-a3e/agent-ae1a3ead3cabd0a4c.jsonl | round 4 (unit B) | simplification/wrap re-probe |
| workflows/wf_1741f4ad-48d/agent-ad91aafe765192f45.jsonl | round 4 (unit C) | sort-aware binding + semantic guardedness |
| workflows/wf_1741f4ad-48d/agent-ac163fe10def094e0.jsonl | round 4 (unit D) | typed parse + stream-aware framing |
| workflows/wf_1741f4ad-48d/agent-aeaa95d99395a0136.jsonl | round 4 (unit E) | nullary/acc-lemma/case-test expansion |
| workflows/wf_1741f4ad-48d/agent-af76260f88f21bcaa.jsonl | round 4 (unit G) | report contract, ordering, scope, batching |
| workflows/wf_1741f4ad-48d/agent-ae1942fd5bd87afee.jsonl | round 5 (unit B) | clusters, wrap wiring, temporal nodes, RawRule entry |
| workflows/wf_1741f4ad-48d/agent-a92e0a97d8d241648.jsonl | round 4 (unit A) | full route surface + state machine |
| workflows/wf_60530b15-4b6/agent-a4bb77314f80cb4c2.jsonl | round 5 (unit A) | del/path + verify families |
| workflows/wf_60530b15-4b6/agent-a8ed0da34ff4e5fe5.jsonl | round 6 (unit B) | record-level group-budget wrap trigger |
| workflows/wf_60530b15-4b6/agent-ab299adf6d8019f6c.jsonl | round 5 (unit E) | staged-mode expansion contract |
| workflows/wf_a3b4feac-a25/agent-acb7bffc97984afe3.jsonl | round 6 (unit E) | bare-nullary decoration guard |
| workflows/wf_a3b4feac-a25/agent-af2c6e9e0d378f62e.jsonl | round 5 (unit D) | summary content + incremental emitter |
| workflows/wf_a3b4feac-a25/agent-a861412dfce110af9.jsonl | round 6 (unit A) | origin-aware shell + StateOps trait |
| workflows/wf_a3b4feac-a25/agent-a7981ceb299f1dfa5.jsonl | round 7 (unit B) | trigger/fill budget split, greedy allocation |
| workflows/wf_3b637271-f7f/agent-*.jsonl (implementers among 14+) | wave 4: B8 (HughesPJ-faithful engine from sanctioned BSD source), A7 (concurrency contract + snapshot/commit dispatch), D6 (diff verdict taxonomy) |
| workflows/wf_a43edc65-4dc/agent-ab519587162798c8b.jsonl | round 7 (unit D, overlapped) | non-diff verdict family (AnalysisCannotBeFinished) |
| workflows/wf_b6a947c0-a82/agent-aac83361400743475.jsonl | round 5 (unit C) | wf corpus-divergence closure: equality-guards + quantifier fusion, sort-clash keying, tail (full gate 71→8 DIFF) |
| workflows/wf_6b16799d-1c9/agent-a6d5afeb848c42e89.jsonl | round 9 (unit B) | fill-width allocation: two-layer trigger/fill model probe-exact 343/343, engine laziness fix, residue characterized (abbreviation-width = caller-input gap) |
| workflows/wf_f9696b2b-75e/agent-abaf7a3894e2dccc3.jsonl | producers round 1 | web producer surface R1: shared HTML skin + center section fragments (324/324 corpus fragments byte-identical incl. envelope; live-probe replays 8/8) |
| workflows/wf_58b2a176-d66/agent-ab3a750508a830592.jsonl | round 10 (unit B) | fill-width laws corrected by probe (sqa=none, C=flat+Σ(elems+1), bonus=⌊n/2⌋+2 cap 4), band-NONE 10.3%→5.83%, CellWidths override interface (probe byte-exactness 525/547) |
| workflows/wf_cdfa493d-262/agent-aebcd06acb5de46a7.jsonl | round 6 (unit C) | wf residual closure: ble list dedup removed (one entry per premise occurrence, source order); pubcap/fact-cap/fact-arity pinned as already-correct; open-side split confirmed via probes |
| workflows/wf_cdfa493d-262/agent-a57fc416f110972cf.jsonl | pretty round 1 | term core + signature block: Doc-based renderer at 110/73, builtin expansion tables oracle-learned, 41 tests, whole-block parity on all 10 round-1 files |
| workflows/wf_910e7687-413/agent-a4262551b04e2c696.jsonl | producers round 2 | R5 path grammar (923k-href harvest, live parse batteries, 40,037-tail round-trip) + R2 west pane (478/478 pane byte-identity; ≤69 escaped-width layout boundary live-bisected) |
| workflows/wf_6c1add5e-f10/agent-a5c3c235083120633.jsonl | round 11 (unit B) | five allocation laws (half-DOWN, recursive occupancy, cap-7 numerator, last-arg bonus gate, tuple-opener hang) + relief pass; families 2/4 dissolved by ?unabbreviate= twins; wrapping 86.3→95.6%, single-cell 100% |
| workflows/wf_e914d4ec-17f/agent-a47c900ac52dd2fdf.jsonl | producers round 3 | R3 proof-tree HTML (case=CHILD/next-qed=PARENT status law, hl_superfluous replayed mode; 478/478 sweep upgraded w/ proof lines produced, live replays 4/4) + R4 welcome/housekeeping (6/6 live byte replays); producer surface R1–R5 complete |
| workflows/wf_a5ef3578-648/agent-a6c440b2693c9355d.jsonl | pretty round 2 | equation-order law (structural, 13 probes refuting shortlex/arity-first) + wide-tuple hang; 12-file signature parity; R2 rule rendering 72/72 blocks byte-identical |
| workflows/wf_bf6a23fd-437/agent-a22a5781796e057ae.jsonl | round 7 (unit C) | per-finding report granularity: full per-topic counting law oracle-pinned (whole-topic/per-rule/per-side/per-fact/per-item/per-lemma; MDC=1/theory); 5 corpus footers reconcile exactly (dir also holds the two session-limit-killed wf_c7ddbc35-082 attempts, empty) |
| workflows/wf_8ba08a93-8eb/agent-a4e016ab960f7a20a.jsonl | round 12 (unit B) | slack law ⌈e/2⌉−1 any-position (retracts round-11 last-gated bonus as relief artifact), ftup occupancy, relief charge min(hd(q+1/3),C); wrapping 95.6→96.45%; ΣC=88 zone PROVEN non-closed-form (terminal) |
| workflows/wf_1c3b9e54-e0c/agent-a67c8657acbf2eb3a.jsonl | pretty round 3 | formula rendering: glyph/paren/binder/wrap laws probe-pinned, safety classifier, restriction+lemma wrappers, guarded-comment frame; 84 restriction + 139 lemma blocks byte-identical over 20 files; R1 AC-op law corrected by corpus witness |
| workflows/wf_e752c3f6-6f4/agent-a76c533bec1fe9206.jsonl | pretty round 4 | process=\"…\" attribute law (round-2 "dropped" belief refuted — SAPIC-free corpus artifact); >)-case proven already-fixed by R3 + app-join probe; stack overflow fixed by iterative Doc-engine rewrite (byte-neutral, 8MB-stack deep-file parity); unit R1–R4 COMPLETE |
| workflows/wf_92d2ed53-daf/agent-ae03098e391b35ba6.jsonl | round 13 (unit B) | edge-layer completion: 11/11 (color,style) vocabulary (census-total, panic-on-unknown = totality proof), EndRef::Info both-ends anchor (port id = node_id − concl − 1, capture-verified); edge-section regeneration 12,022/12,022 byte-exact corpus-wide |
| workflows/wf_a320cb04-549/agent-a2d8cbdd7c2f76260.jsonl | pretty round 5 | batch echo completed: Restriction.expanded field (statement=macro form, comment=expanded — probed both ways), macros always-break vcat law, theory-frame assembly + predicates margin-0 splice; whole-echo parity 29/29 captures |

Corresponding workspaces (../wellformedness, ../graphdot, ../weblayer) hold
each agent's QUERIES.log, BEHAVIOR.md, REPORT*.md and code — cross-reference
the transcripts against those logs.

## Similarity-audit transcripts (both-sides reviewers, exempt from access rules)

| Path | Role |
|------|------|
| workflows/wf_7197ba11-003/agent-*.jsonl (3) | round-1 audits: wf / graph / web (verdicts: pass, 0 findings) |
| standalone-agents/agent-aac7737a5d3b7d6e7.jsonl | wf round-2 incremental audit (pass, 0) |
| standalone-agents/agent-ac1b65c4df499573b.jsonl | graph+web round-2 incremental audit (pass, 0) |
| workflows/wf_bfd5a6a0-31b/agent-a44d06c824ea88b18.jsonl | round-3 audits: console/macros/injective/derivcheck (all pass, 0) |
| workflows/wf_bfd5a6a0-31b/agent-adb12daa8ab777100.jsonl | round-3 audits: weblayer + graphdot deltas (pass, 0) |
| workflows/wf_bfd5a6a0-31b/agent-a69817b344aadd7bd.jsonl | round-3 audit: wellformedness delta (pass, 0) |
| workflows/wf_62e2c0ef-a3e/agent-af3d04b49cd4b056d.jsonl | round-4 audit: graphdot re-probe delta (pass, 0; notes clean side's "levels inert" conclusion contradicts the source — affirmative non-access evidence) |
| workflows/wf_1741f4ad-48d/agent-a*.jsonl (6 auditor transcripts among 14) | round-4/5 per-unit delta audits: C/D/E/G/B/A (all pass, 0 findings; D audit notes the clean flag-precedence order DIVERGES from the source Either-applicative order — affirmative non-access evidence) |
| workflows/wf_60530b15-4b6/agent-{ae835430920982249,aa7a475a42e7f0a16,a13ca9ba2bbcd5182}.jsonl | round-5/6 delta audits: E / A / B (all pass, 0 findings) |
| workflows/wf_a3b4feac-a25/agent-{a9df6213a26883a9d,a49c583ac48623111,a1250a539cbe74da9,aab79db7bb6b49c3c}.jsonl | round-5/6/7 delta audits: E / D / A / B (all pass, 0 findings) |
| workflows/wf_b6a947c0-a82/agent-a351c5451327cd399.jsonl | wf round-5 delta audit (pass, 0 findings; closes the round-4 equality-guard advisory — clean side previously under-claimed vs the source, reached parity by probing) |
| workflows/wf_6b16799d-1c9/agent-aef2578a65838b522.jsonl | graphdot round-9 delta audit (pass, 0 Dot.hs-resemblance findings; laziness = sanctioned BSD semantics; every fitted constant probe-traced, disjoint from source's 100/1.3/130/30; one non-blocking behavioral redo: reconcile sqa correction magnitude/placement) |
| workflows/wf_f9696b2b-75e/agent-aad7d84bc51151972.jsonl | producers round-1 dual audit (pass; similarity: 0 violations, expression diverges materially from upstream; hygiene: finding PH-1 — scaffold SPEC/README cited upstream file paths, rewrite applied open-side, no contamination of sealed work) |
| workflows/wf_58b2a176-d66/agent-abc902c75185add51.jsonl | graphdot round-10 delta audit (pass; round-9 sqa advisory resolved consistently, evidence-backed; size laws probe-traced to the column, refuting round-9's own 2n−4; no Dot.hs convergence) |
| workflows/wf_cdfa493d-262/agent-a6d3c6533fd3d2c31.jsonl | wf round-6 delta audit (pass; identifier constellation disjoint from the Haskell; boundary bytes = merger) |
| workflows/wf_cdfa493d-262/agent-a5fb0254d4385f4a6.jsonl | pretty round-1 delta audit (pass; doc.rs byte-identical to the audited BSD port — GPL wrapper only delegates; term core brackets from the outside, no copied expression) |
| workflows/wf_910e7687-413/agent-a627c1987001ec5ff.jsonl | producers round-2 delta audit (pass; numeric grammar reconstructs GHC-reads behavior upstream never expresses; zero identifier overlap; PH-1 confirmed resolved; 3 non-blocking notes) |
| workflows/wf_6c1add5e-f10/agent-aa34eecb3a83aa639.jsonl | graphdot round-11 delta audit (pass, 0 violations; half-DOWN diverges from source's half-even; structural divergence WIDENED — two-pass/shape-terms vs single-pass/shape-blind; every law probe-traced) |
| workflows/wf_e914d4ec-17f/agent-aa69aa0d5168094f4.jsonl | producers round-3 delta audit (pass, 0 similarity; pointed omission of upstream vestiges a copier would carry; hardest laws corpus-refuted then live-forced; hygiene clean). agent-a3655b013a80004f3 = first audit attempt, killed by session limit (empty) |
| workflows/wf_a5ef3578-648/agent-ad4db13353b8f69dd.jsonl | pretty round-2 delta audit (pass; ordering law diverges from source Ord in every non-observable slot — omits idx-sort-name and position tiebreak, invents PairTail; AC ops pointedly not unified onto one helper). agent-a42415d79a2b4cfbd = first audit attempt, killed by session limit (empty) |
| workflows/wf_bf6a23fd-437/agent-a8f291ecb5218b5d7.jsonl | wf round-7 delta audit (pass; granularity = merger — footer N is boundary-observable; disjoint identifiers; affirmative divergence: preamble carried in first finding's body vs reference's Topic string) |
| workflows/wf_8ba08a93-8eb/agent-ae5198dea2befef01.jsonl | graphdot round-12 delta audit (pass, 0 violations; 1/3 constant OUTPUT-derived within the byte-pinned window, absent from source; divergence widened; the honest false-wrap regression into the proven-terminal zone = evidence against copying) |
| workflows/wf_1c3b9e54-e0c/agent-adf2b133dc2654170.jsonl | pretty round-3 delta audit (pass; upstream has NO precedence table — always-parens matched via genuine discriminator probe; binder allocation divergent from de-Bruijn machinery; helper decomposition divergent; 5 non-blocking advisories) |
| workflows/wf_e752c3f6-6f4/agent-a815108459a348d7e.jsonl | pretty round-4 delta audit (pass; process attr byte-forced by probe capture, clean prints verbatim snippet vs upstream re-render machinery; stack fix = explicit-stack iteration on the BSD engine — GPL wrapper has no recursion to copy; owned-teardown Drop exists on neither side) |
| workflows/wf_92d2ed53-daf/agent-a35e46f330272a8d9.jsonl | graphdot round-13 delta audit (pass; census independently reproduced to the count; style-first byte order of green/dotted = capture-forced merger; source has no edge-style enum; "info" is sealed coinage — source keys the cell Nothing) |
| workflows/wf_a320cb04-549/agent-a8f460a120615533d.jsonl | pretty round-5 delta audit (pass; frame traversal does NOT mirror upstream's 8-handler item-fold — 7-arm match over erasure-driven enum, no hoisting; macros vcat byte-forced by falsification probe; resolved the apparent $$-vs-col-8 discrepancy showing source and oracle agree) |

Findings text: ../wellformedness/AUDIT.md, ../graphdot/AUDIT.md, ../weblayer/AUDIT.md,
../console/AUDIT.md, ../macros/AUDIT.md, ../injective/AUDIT.md, ../derivcheck/AUDIT.md.

## Dirty-room and campaign-research transcripts

| Path | Role |
|------|------|
| main-session-dirty-room.jsonl | the orchestrating session: header generation, oracle/corpus construction, interoperability-header extraction, integration fixes, all audits as run |
| workflows/wf_610d94ab-e29/ | dirty-room integration agent (adapters only; see ../INTEGRATION_REPORT.md) |
| workflows/wf_62e2c0ef-a3e/agent-ac4e655d83d5c3d56.jsonl | dirty-room integrator: units A+B re-sync (rewire blocked, see ../INTEGRATION_REPORT.md) |
| workflows/wf_62e2c0ef-a3e/agent-a2eac17c5efa35c13.jsonl | dirty-room integrator: units C–G (partial; keeps reported per protocol) |
| workflows/wf_1741f4ad-48d/ (2 integrator transcripts among 14) | round-4/5 integrators: theory C-swap+delete (check_terms.rs removed, 134->133) and server A/B (blocked; edit-form closure byte-verified) |
| workflows/wf_60530b15-4b6/agent-{aba0d9334e4c1f215,ae21d5199beeba498,a59a476a912c12eee,ab0787ce52d9ab2af}.jsonl | wave-2 dirty room: G swap COMPLETE (deriv_check.rs deleted, 10/10 byte gate); D partial (version-stream swap; parse/framing blocked); E+B integrator (both kept, blockers refined to byte-precision); A integrator (del/verify re-synced; adoption blocked on origin-shell + async-state) |
| workflows/wf_a3b4feac-a25/agent-{a66029e08d55564f9,acb62e977f9aaf9c3}.jsonl | wave-3 integrators: E SWAPPED (ported macro-expansion driver deleted, live byte-verified vs HS); D summary/emitter landed, driver swap kept (9-verdict taxonomy gap); B kept (fill is fillSep/fits — Ack cell diverges at byte 318); A kept (async-serialization design conflict, code-evidenced) |
| workflows/wf_b6a947c0-a82/agent-ac6e3390a695e1a84.jsonl | open-side integrator: wf round-5 re-sync (checks.rs/pretty.rs, mechanical recipe), full wf_gate 348 MATCH/71 DIFF → 411/8, 0 regressions, all suites green |
| workflows/wf_f9696b2b-75e/agent-ab36f0d758d273636.jsonl | open-side scaffolder: weblayer/producers/ cluster (SPEC R1–R5 decomposition, oracle wrapper fixing the stock hs_server.sh --port bug, expression-stripped interface, 44 round-1 byte targets) |
| workflows/wf_cdfa493d-262/agent-aaf0e68eb9e4e9a0a.jsonl | open-side integrator: wf round-6 closure — re-sync + preamble-map fixes + sapic_public_names_report tuple-order bugfix; full wf_gate 419 MATCH / 0 DIFF (unit at full-corpus zero) |
| workflows/wf_cdfa493d-262/agent-a281e419853d7d5f5.jsonl | open-side integrator: pretty R1 vendored (crates/tamarin-theory/src/pretty_clean/, headerless, reverse-transform byte-identical) + adapter; both adoptions kept ported with byte-precision blockers (trial signature swap 401/403; equation order + tuple wrap); all gates green, header delta 0 |
| workflows/wf_c87ee431-f2d/agent-aea2058e09b9ed188.jsonl | open-side integrator: graph round-10 occupancy adapter — REFUTED at corpus scale (internal widths regress the gate: wrapping 86.26%→83.9%, FIXED 372/BROKEN 5635); display-flat trigger 99.63% accurate; 4 ranked round-11 families with first-diverging-byte examples; kept ported, header delta 0 |
| workflows/wf_7081ec9f-f00/agent-*.jsonl | open-side integrator: graph round-11 re-sync + measurement — whole-payload 52.58%→63.28%, records 89.83%→94.17%, diverging cells −54.5%; 3 ranked fill families remain; 63.3% is the CEILING for cell-layout-only routing (full-generate blocked on LNTerm adapter + dialect oracle); kept ported |
| workflows/wf_bf6a23fd-437/(gate agent).jsonl | open-side integrator: wf round-7 re-sync (checks/report) + 7 header-less topic registrations w/ empty preamble; wf_gate 419/0 unchanged, footer gate 5/5 MATCH (3/14/14/5/6) — wellformedness unit at FULL-OUTPUT parity (block + footer) under --prove corpus-wide |
| workflows/wf_56cc20c9-d78/agent-*.jsonl | open-side integrator: producers integration — vendored producers/ + adapters; main/help routed byte-identical (triple-proven); ZERO deletions (all four producer files interleave solver core — ProofState defn, sources/saturation, axum plumbing, Method grammar); author-erasure NET ZERO DOUBLY: pseudonymous/web authors spread across the entire solver/term/graph/pretty core — no web deletion can retire them; all gates green, headers 133 (delta 0) |
| workflows/wf_6384b98e-ff2/agent-*.jsonl | open-side integrator: pretty round-2 — SIGNATURE SWAP ADOPTED byte-green (403/16 exact, zero new divergence; ported render_signature kept only for web_signature_block); rule route attempted → 88 new DIFFs → reverted with 3 witnesses (SAPIC process= attr 79 files; closing-bracket wrap 5; clean-render stack overflow 4); all gates green, headers 133 (delta 0) |
| workflows/wf_a589ffe6-62f/agent-*.jsonl | open-side measurement: graph FULL-generate (no adoption) — byte 3.97% whole-payload BUT decomposed: info-port anchor gap (94% of payloads) + 3 missing edge styles + fill ceiling 65.4% non-edge; SEMANTIC 45.8%→75.7% (styles closed)→91.7% (bracket-normalized); KEY: the tree's live serializer is ALREADY byte 0/100 semantic 100/100 vs HS (compact viz.js dialect) — the byte standard never held on the served graph route; harness archived in task-results/ |
| workflows/wf_af332efc-d91/agent-*.jsonl | open-side integrator: pretty endgame — RULE + LEMMA routes ADOPTED byte-green (incl. --prove parity 13899/13899 UM_three_pass, 250/250 Tutorial); restriction/macros/frame blocked (3 sealed witnesses); KEY: web panes render width-100 HtmlDoc w/ hl_* spans — a different surface keeping every ported renderer alive; deletions 0, author delta 0; batch/web width gate added; all gates green |
| workflows/wf_a25cc7a0-1bd/agent-*.jsonl | open-side integrator: pretty round-5 — RESTRICTION + MACROS + PREDICATES routes ADOPTED byte-green (403/16 exact, 0 non-spore DIFFs); frame DECLINED with cause (clean models the gate-stripped echo; CLI must emit wf/stamp/config blocks) — block-level routing is the end state; all gates green, headers 133 |
| workflows/wf_53f48d60-3ef/ | licensing research (GPL doctrine/precedent/feasibility/literal-copy) |
| workflows/wf_8b8fd87b-72e/ | uncited-file derivation audit + institutional-ownership research |
| workflows/wf_5ec90b24-b26/ | tail-author affiliations, triviality classification, rewrite scoping |
| standalone-agents/agent-a9c7edd67d24711cc.jsonl | contributor-provenance analysis (withheld from public mirror) |

## Supporting material

- launch-scripts/ — the workflow scripts (contain the verbatim prompts given to
  every agent; prompts are also embedded in each transcript's first message).
  Exception: the round-3 implementation wave (wf_7f807521-670) has no persisted
  launch script; its prompts survive only in the transcripts' first messages.
- task-results/ — result summaries as returned to the orchestrator (rescued
  from volatile /tmp).
- journal.jsonl inside each workflow dir — the workflow engine's own record of
  each agent's launch and structured return value.
- audit/transcript_audit.py — the audit tool; audit/cleanroom_audit_results.txt
  — its final run over all clean-room transcripts.

## Integrity and limitations

- SHA256SUMS (300 files at publication; refreshed to 727 files the same day
  after the round-3/4 waves — every original entry retained, new cluster
  workspaces and transcripts added) fixes the archive's content as of 2026-07-17. To
  timestamp-anchor it, commit this directory (or just SHA256SUMS) to a git
  repository and/or push to a remote — the commit chain provides an ordered,
  hard-to-backdate record. OpenTimestamps is a stronger option if wanted.
- These transcripts are harness-produced records under the operator's control,
  not third-party attestations. Their evidentiary weight comes from being
  contemporaneous, complete (every tool call), internally consistent with the
  workspaces' QUERIES.log/BEHAVIOR.md, and fixed by hash before any dispute.
- One known caveat is documented in ../PROTOCOL.md: the implementing model's
  training data may contain the GPL source; the mitigations (no-recall
  instruction, independent both-sides similarity audits, divergence evidence
  recorded in the AUDIT.md files) are part of this record.

## Public-mirror note (added at publication)

The public mirror at github.com/kmilner/tamarin-cleanroom withholds the
campaign-research transcripts, the orchestrating-session transcript, and the
contributor-analysis outputs (privacy: they de-pseudonymize contributors).
Withheld files remain in the local archive and are integrity-anchored by
their entries in SHA256SUMS, which is published in full.
