#!/usr/bin/env bash
# scrape.sh FILE MODE PORT
#   FILE : path relative to oracle/examples (e.g. classic/NSLPK3.spthy)
#   MODE : trace | diff
#   PORT : TCP port for this isolated interactive server
#
# Launches an isolated tamarin-prover interactive server on the single theory,
# blocks until it finishes closing (that is when injective instances / rules are
# computed), then captures the rendered rules page(s):
#   trace -> /thy/trace/<idx>/main/rules       (has the injective-instances section)
#   diff  -> /thy/equiv/<idx>/main/diffrules   (diff mode: no injective section)
# Raw JSON responses are saved under corpus_html/. Waiting is done by blocking on
# the log (tail -f | grep), never by a sleep-poll loop.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
EX=/home/kamilner/tamarin-cleanroom/injective/oracle/examples
ORACLE=/home/kamilner/tamarin-cleanroom/injective/oracle/hs_oracle.sh
SCRATCH=/tmp/claude-1000/-home-kamilner-tamarin-rs/57f8e1bc-d495-4b36-93c0-033347d5034a/scratchpad
HTML="$HERE/corpus_html"; LOGS="$HTML/logs"
mkdir -p "$HTML" "$LOGS"

FILE="$1"; MODE="$2"; PORT="$3"
slug="$(echo "$FILE" | sed 's#/#__#g; s#\.spthy$##')"
wd="$SCRATCH/wd_$slug"; rm -rf "$wd"; mkdir -p "$wd"
cp "$EX/$FILE" "$wd/"
# Some theories set `heuristic: o "./oracle-NAME"`; tamarin spawns that script at
# load time. It only ranks goals during proof search and has no bearing on the
# injective-instance precomputation, so a no-op passthrough stub lets the theory
# close in an isolated workdir. (Documented in the corpus note.)
for on in $(grep -oE 'oracle-[A-Za-z0-9._-]+' "$wd/$(basename "$FILE")" | sort -u); do
  printf '#!/bin/sh\ncat >/dev/null\n' > "$wd/$on"; chmod +x "$wd/$on"
done
log="$LOGS/$slug.log"; : > "$log"
status="$HTML/$slug.status"

extra=""; route="trace"; page="main/rules"
if [ "$MODE" = "diff" ]; then extra="--diff"; route="equiv"; page="main/diffrules"; fi

echo "ORACLE: hs_oracle.sh interactive <wd:$FILE> --port=$PORT --no-logging $extra"
ORACLE_TIMEOUT=900 nohup "$ORACLE" interactive "$wd" --port=$PORT --no-logging $extra > "$log" 2>&1 &
spid=$!

# Block until the theory closes (or the server dies / 480s cap), no sleep-poll.
if timeout 480 tail -n +1 --pid="$spid" -f "$log" 2>/dev/null \
     | grep -m1 -q "Theory closed"; then
  ready=closed
else
  ready=notclosed
fi

if [ "$ready" != "closed" ]; then
  echo "STATUS=$slug FAIL($ready)"; echo "FAIL:$ready" > "$status"
  kill "$spid" 2>/dev/null; wait "$spid" 2>/dev/null; rm -rf "$wd"; exit 0
fi

# Large theories register in the web server a little after logging "Theory
# closed"; poll the root for the theory index (bounded), do not sleep-poll hard.
idxs=""
for ((k=0;k<60;k++)); do
  root="$(curl -s --retry 5 --retry-connrefused --retry-delay 1 -m 30 "http://127.0.0.1:$PORT/")"
  idxs="$(printf '%s' "$root" | grep -oE "/thy/$route/[0-9]+" | grep -oE '[0-9]+$' | sort -un)"
  [ -n "$idxs" ] && break
  sleep 1
done
if [ -z "$idxs" ]; then
  echo "STATUS=$slug FAIL(no-index)"; echo "FAIL:no-index" > "$status"
  kill "$spid" 2>/dev/null; wait "$spid" 2>/dev/null; rm -rf "$wd"; exit 0
fi

n=0
for idx in $idxs; do
  out="$HTML/$slug.$route$idx.json"
  curl -s --retry 5 --retry-connrefused --retry-delay 1 -m 40 \
       "http://127.0.0.1:$PORT/thy/$route/$idx/$page" > "$out"
  echo "ORACLE: GET /thy/$route/$idx/$page  ($FILE) -> $(basename "$out")"
  n=$((n+1))
done
echo "OK:$route:$n-idx" > "$status"
echo "STATUS=$slug OK($route,$n)"
kill "$spid" 2>/dev/null; wait "$spid" 2>/dev/null; rm -rf "$wd"
exit 0
