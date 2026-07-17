# Unit D: CLI/console surface

Build crate cli-clean in workspace/ reproducing the tamarin-prover BINARY's
command-line text surface byte-for-byte, from probing oracle/hs_oracle.sh
(passes arbitrary args to the black-box binary; examples in oracle/examples/).
Cover: --help (global and per-mode; enumerate modes from help output),
--version, unrecognized flag/argument error messages, missing/unreadable file
errors, the batch-mode output FRAMING around a processed theory (banner,
"analyzed: ..." summary block, timing/appendix lines — treat the theory
pretty-print itself as an opaque payload slot), and flag parsing semantics
(which flags take values, defaults shown in help). Deliverable: parse(argv)
-> structured command | error-text, plus render_help/render_version/framing
functions. Own design; no interface header exists. BEHAVIOR.md + QUERIES.log
+ tests with captured fixtures; REPORT.md + 10-line summary.
