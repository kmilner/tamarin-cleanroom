#!/usr/bin/env bash
# Split-stream probe: runs the reference binary capturing stdout and stderr
# separately into captures/<label>.out.txt and captures/<label>.err.txt, and
# prints the exit code. Usage: split_probe.sh <label> [args...]
# The binary path mirrors oracle/hs_oracle.sh (compiled black-box binary).
set -u
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
BIN="/home/kamilner/tamarin-rs/tamarin-prover-testing/.stack-work/install/x86_64-linux-tinfo6/ec0cb11b1bfcf8776d45e0357bbc6d6ff2077f9222735af22115429c8cdfcef1/9.6.7/bin/tamarin-prover"
CAP="/home/kamilner/tamarin-cleanroom/console/workspace/captures"
label="$1"; shift
timeout "${ORACLE_TIMEOUT:-120}" "$BIN" "$@" 1>"$CAP/$label.out.txt" 2>"$CAP/$label.err.txt"
rc=$?
echo "exit=$rc"
echo "--- $label.out.txt ($(wc -c <"$CAP/$label.out.txt") bytes) ---"
cat "$CAP/$label.out.txt"
echo "--- $label.err.txt ($(wc -c <"$CAP/$label.err.txt") bytes) ---"
cat "$CAP/$label.err.txt"
