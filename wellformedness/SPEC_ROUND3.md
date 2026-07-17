# Round 3 (Unit C): complete the wellformedness surface

Extend workspace/wf-clean. Oracle: oracle/wf_oracle.sh <file> [flags].
1. "Formula terms" FULL coverage: the oracle flags formula terms containing
   reducible user-defined functions (theories with `equations:`). Your new
   entry point takes the theory PLUS a caller-supplied set of reducible
   function symbol names (the caller computes it; you only consume it).
   Probe with user-equation theories; byte-match messages incl. the de Bruijn
   `Bound N` renderings you noted in round 1.
2. Formula guardedness: close the round-1 gaps (multi-line wrapped formula
   rendering; precise nested-subformula selection) by probing formulas the
   oracle wraps across indented lines.
3. Probe whether a "Fact capitalization issues" topic exists (hint: fact
   names differing only in case across rules) and implement byte-exact.
4. Close remaining round-1/2 documented gaps that are AST-derivable:
   fsep wrapping of wide fact lists (~col 80), fresh-const list wrapping,
   lemma-fact arity conflicts, multi-group public-names separators.
OUT OF SCOPE: Message Derivation / Derivation Checks (separate unit).
Update BEHAVIOR.md/QUERIES.log; tests per item; REPORT3.md + 10-line summary.
