#!/usr/bin/env bash
# Materialize the round-1 byte targets (sub-target R1: theory-view CENTER
# section fragments) into round1/targets/, sliced straight from the captured
# corpus by oracle/extract_fragments.py in a SINGLE corpus pass. Touches ONLY
# the captured corpus (the sanctioned channel) — no live crawl, no source tree.
# Re-runnable.
#
#   ./fetch_targets.sh        # (re)materialize targets/ from families.tsv
set -u
here=$(cd "$(dirname "$0")" && pwd)
EX="$here/../oracle/extract_fragments.py"
OUT="$here/targets"
FAMILIES="main/message,main/rules,main/tactic,main/help"

# Collect the curated labels (col 2) into one comma list -> one manifest pass.
labels=$(awk -F'\t' '!/^#/ && NF>=2 && $2!="" {printf "%s%s", sep, $2; sep=","}' "$here/families.tsv")
[ -z "$labels" ] && { echo "no labels in families.tsv" >&2; exit 2; }

rm -rf "$OUT"; mkdir -p "$OUT"
python3 "$EX" extract "$FAMILIES" "$OUT" --only "$labels"

files=$(find "$OUT" -maxdepth 1 \( -name '*.html' -o -name '*.txt' \) | wc -l)
echo "materialized $files center-fragment target files in targets/  (families: $FAMILIES)"
