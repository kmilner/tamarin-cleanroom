#!/usr/bin/env bash
# Materialize the Haskell reference THEORY ECHO for each round-3 target into
# round3/targets/<safe>.hs.txt, using the sanctioned HS oracle only. Run from
# inside the sealed workspace; touches only the HS binary (via pretty_oracle.sh)
# and the corpus .spthy inputs — both permitted materials.
set -u
here=$(cd "$(dirname "$0")" && pwd)
CORPUS="${CORPUS:-/home/kamilner/tamarin-rs/tamarin-prover/examples}"
FLAGS_MAP="${FLAGS_MAP:-/home/kamilner/tamarin-rs/scripts/file_flags.tsv}"
ORACLE="$here/../oracle/pretty_oracle.sh"
mkdir -p "$here/targets"
n=0
while IFS=$'\t' read -r _tags REL; do
  case "$_tags" in \#*|"") continue;; esac
  f="$CORPUS/$REL"
  [ -f "$f" ] || { echo "MISSING $REL" >&2; continue; }
  fl=$(awk -F'\t' -v r="$REL" '!/^#/ && $1==r {print $2}' "$FLAGS_MAP"); fl=${fl//@cd/}
  safe=$(printf '%s' "$REL" | tr '/' '_')
  # pretty_oracle.sh runs HS no-prove and prints the extracted theory echo.
  # shellcheck disable=SC2086
  "$ORACLE" "$f" $fl > "$here/targets/$safe.hs.txt" && n=$((n+1))
done < "$here/families.tsv"
echo "wrote $n HS reference theory-echo blocks to targets/"
