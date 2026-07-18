#!/usr/bin/env bash
# Black-box oracle for the WEB PRODUCER SURFACE clean-room.
#
# Serves ONE .spthy in the sanctioned Haskell interactive web UI so the sealed
# room can curl any route and observe the exact HTTP response body it must
# reproduce. Live probing complements the captured corpus (see
# ../round1/ + oracle/extract_fragments.py); use it to pin fragment structure,
# link targets and section headers a single capture leaves ambiguous.
#
# Usage:
#   hs_server.sh start <file.spthy> [port]   # boot + block until ready; prints URL
#   hs_server.sh probe <port> <path>         # curl one route, print the body
#   hs_server.sh stop  <port>                # kill the server + free the port
#   hs_server.sh smoke <file.spthy> [port]   # start, probe /overview + /main, stop
#
# Ports 3100-3199 belong to this cluster (default 3100). Everything below is
# derived from black-box behavior; no tamarin-rs source is read — the only
# tamarin-rs touch is EXECUTING this sanctioned binary.
#
# Why this wrapper exists (do not "simplify" back): the stock weblayer
# hs_server.sh passes `--port <n>` with a SPACE, which the prover's CmdArgs
# parser reads as the positional WORKDIR (`directory '<n>' does not exist`) and
# the server dies silently after a fixed `sleep 4`. This wrapper (a) uses
# `--port=<n>` (equals, like scripts/web_parity.sh), (b) POLLS for readiness
# instead of sleeping (maude boot + elaboration routinely take >10s), and
# (c) frees the port on stop (pid + process-group + listener fallback).

set -u

# ---- OOM guards (preserve; a runaway prover must be the kill target) --------
echo 1000 > /proc/self/oom_score_adj 2>/dev/null || true
ulimit -v 25165824 2>/dev/null || true            # 24 GB address-space cap

export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH" # maude / dot live here
HS="${HS_PATH:-/home/kamilner/tamarin-rs/tamarin-prover-testing/.stack-work/install/x86_64-linux-tinfo6/ec0cb11b1bfcf8776d45e0357bbc6d6ff2077f9222735af22115429c8cdfcef1/9.6.7/bin/tamarin-prover}"
READY_TIMEOUT="${READY_TIMEOUT:-90}"
DERIVCHECK_TIMEOUT="${DERIVCHECK_TIMEOUT:-30}"    # match web_parity.sh (5s default expires on ~12 files)

pidfile() { echo "/tmp/producers_hs_$1.pid"; }
wdfile()  { echo "/tmp/producers_hs_$1.wd"; }
logfile() { echo "/tmp/producers_hs_$1.log"; }

free_port() {
  local port="$1" pid
  pid=$(cat "$(pidfile "$port")" 2>/dev/null || true)
  if [ -n "$pid" ]; then
    kill -TERM "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true
    sleep 1
    kill -KILL "-$pid" 2>/dev/null || kill -KILL "$pid" 2>/dev/null || true
  fi
  # Fallback: anything still bound to the port (reaps orphaned maude children).
  if command -v fuser >/dev/null 2>&1; then
    fuser -k "${port}/tcp" 2>/dev/null || true
  fi
  local wd; wd=$(cat "$(wdfile "$port")" 2>/dev/null || true)
  [ -n "$wd" ] && [ -d "$wd" ] && rm -rf "$wd"
  rm -f "$(pidfile "$port")" "$(wdfile "$port")"
}

start() {
  local f="$1" port="${2:-3100}"
  [ -f "$f" ] || { echo "no such file: $f" >&2; return 2; }
  [ -x "$HS" ] || { echo "no HS binary at $HS" >&2; return 2; }
  free_port "$port"                                # clean any stale bind first
  local wd; wd=$(mktemp -d); mkdir -p "$wd/thy"; cp "$f" "$wd/thy/"
  echo "$wd" > "$(wdfile "$port")"
  # setsid → own process group so `free_port` can signal the whole tree.
  # NOTE: `--port=<n>` (equals) is load-bearing; a space breaks arg parsing.
  setsid "$HS" interactive "$wd/thy" --port="$port" \
      --derivcheck-timeout="$DERIVCHECK_TIMEOUT" >"$(logfile "$port")" 2>&1 &
  echo $! > "$(pidfile "$port")"
  local pid; pid=$(cat "$(pidfile "$port")")
  local i
  for ((i=0; i<READY_TIMEOUT; i++)); do
    if curl -sf -o /dev/null "http://127.0.0.1:$port/"; then
      echo "ready: http://127.0.0.1:$port/  (pid $pid, ~${i}s)"
      return 0
    fi
    kill -0 "$pid" 2>/dev/null || { echo "server died; log:" >&2; tail -5 "$(logfile "$port")" >&2; free_port "$port"; return 2; }
    sleep 1
  done
  echo "not ready after ${READY_TIMEOUT}s; log:" >&2; tail -5 "$(logfile "$port")" >&2
  free_port "$port"; return 2
}

case "${1:-}" in
  start) shift; start "$@";;
  stop)  free_port "${2:?port}"; echo "stopped $2";;
  probe) curl -s "http://127.0.0.1:${2:?port}${3:?path}";;
  smoke)
    f="${2:?file.spthy}"; port="${3:-3100}"
    start "$f" "$port" || exit 2
    echo "--- /thy/trace/1/overview/help (title) ---"
    curl -s "http://127.0.0.1:$port/thy/trace/1/overview/help" | grep -oE '<title>[^<]*</title>' | head -1
    echo "--- /thy/trace/1/main/message (first 200 bytes) ---"
    curl -s "http://127.0.0.1:$port/thy/trace/1/main/message" | head -c 200; echo
    free_port "$port"; echo "stopped $port (port freed)";;
  *) sed -n '2,33p' "$0"; exit 1;;
esac
