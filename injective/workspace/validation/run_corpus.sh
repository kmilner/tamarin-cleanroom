#!/usr/bin/env bash
# run_corpus.sh [PAR] [LISTFILE]
# Scrapes the rendered rules page for every theory listed in LISTFILE (default
# all_files.txt) via an isolated interactive server, PAR servers in flight.
# Diff theories (diff_files.txt) are loaded with --diff and scraped at
# /main/diffrules; all others at /main/rules. Each scrape.sh launches+waits+kills
# its own server, so this whole script is safe to run detached in the background.
# NOTE: never use pkill here — it matches the harness process group and kills it.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
PAR="${1:-6}"
LIST="${2:-$HERE/all_files.txt}"
mapfile -t ALL < "$LIST"
mapfile -t DIFF < "$HERE/diff_files.txt"
is_diff(){ local f="$1"; for d in "${DIFF[@]}"; do [ "$d" = "$f" ] && return 0; done; return 1; }

port=3700
running=0
for f in "${ALL[@]}"; do
  [ -z "$f" ] && continue
  if is_diff "$f"; then mode=diff; else mode=trace; fi
  echo ">>> launch $mode port=$port $f"
  bash "$HERE/scrape.sh" "$f" "$mode" "$port" >> "$HERE/scrape_run.log" 2>&1 &
  port=$((port+1))
  running=$((running+1))
  if [ "$running" -ge "$PAR" ]; then wait -n; running=$((running-1)); fi
done
wait
echo "=== all scrapes finished ==="
cat "$HERE"/corpus_html/*.status 2>/dev/null | sort | uniq -c
