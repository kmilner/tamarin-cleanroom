#!/usr/bin/env bash
# Unit E acceptance: Rust direct-expansion checks + oracle byte-parity fixtures.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
EX="$HERE/../oracle/examples"
FX="$HERE/fixtures"
rc=0

echo "== cargo test (direct expansion checks) =="
( cd "$HERE/macro-clean" && cargo test --quiet ) || rc=1

echo
echo "== oracle byte-parity fixtures (rule AC-variant: macro vs hand-inlined) =="
pairs=(
  "$FX/capture_macro.spthy|$FX/capture_expanded.spthy"
  "$FX/chain3_macro.spthy|$FX/chain3_expanded.spthy"
  "$FX/over_u3_macro.spthy|$FX/over_u3_expanded.spthy"
  "$FX/sortmatch_macro.spthy|$FX/sortmatch_expanded.spthy"
  "$FX/bare_nullary_macro.spthy|$FX/bare_nullary_expanded.spthy"
  "$EX/regression/trace/issue777.spthy|$FX/issue777_expanded.spthy"
  "$EX/features/macros/MacroInLemmasAndRestrictions.spthy|$FX/lemmas_expanded.spthy"
)
for p in "${pairs[@]}"; do
  m="${p%%|*}"; e="${p##*|}"
  printf '  %-28s ' "$(basename "$m")"
  if bash "$HERE/byteparity.sh" "$m" "$e" >/dev/null 2>&1; then echo OK; else echo FAIL; rc=1; fi
done

echo
echo "== oracle formula-parity fixtures (lemma/acc-lemma/case-test guarded forms) =="
fpairs=(
  "$EX/features/macros/MacroInLemmasAndRestrictions.spthy|$FX/lemmas_expanded.spthy"
  "$FX/casetest_macro.spthy|$FX/casetest_expanded.spthy"
  "$FX/acclemma_macro.spthy|$FX/acclemma_expanded.spthy"
  "$FX/acc_both_macro.spthy|$FX/acc_both_expanded.spthy"
)
for p in "${fpairs[@]}"; do
  m="${p%%|*}"; e="${p##*|}"
  printf '  %-28s ' "$(basename "$m")"
  if bash "$HERE/formula_parity.sh" "$m" "$e" >/dev/null 2>&1; then echo OK; else echo FAIL; rc=1; fi
done

echo
[ $rc -eq 0 ] && echo "ALL PASS" || echo "FAILURES (rc=$rc)"
exit $rc
