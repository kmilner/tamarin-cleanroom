#!/usr/bin/env bash
# Materialize the Haskell reference wf-block for each of the 71 target theories
# into round5/targets/<safe>.hs.txt, using the sanctioned HS oracle only.
# Run this from inside the sealed workspace; it touches only the HS binary and
# the corpus .spthy inputs (both permitted materials).
set -u
here=$(cd "$(dirname "$0")" && pwd)
CORPUS="${CORPUS:-/home/kamilner/tamarin-rs/tamarin-prover/examples}"
FLAGS_MAP="${FLAGS_MAP:-/home/kamilner/tamarin-rs/scripts/file_flags.tsv}"
ORACLE="$here/../oracle/wf_oracle.sh"
mkdir -p "$here/targets"
wf_block() {
  awk '
    /^\/\* All wellformedness checks were successful\. \*\/$/ { print; next }
    /^WARNING: the following wellformedness checks failed!$/  { f=1 }
    f { print }
    f && /^\*\/$/ { f=0 }'
}
while read -r REL; do
  f="$CORPUS/$REL"
  [ -f "$f" ] || { echo "MISSING $REL" >&2; continue; }
  fl=$(awk -F'\t' -v r="$REL" '!/^#/ && $1==r {print $2}' "$FLAGS_MAP")
  fl=${fl//@cd/}
  safe=$(printf '%s' "$REL" | tr '/' '_')
  # wf_oracle.sh runs the HS binary (no --prove needed — wf prints at load).
  "$ORACLE" "$f" $fl 2>/dev/null | wf_block > "$here/targets/$safe.hs.txt"
done < "$here/wf_gate_diffs.txt"
echo "wrote $(ls "$here/targets" | wc -l) HS reference blocks to targets/"
