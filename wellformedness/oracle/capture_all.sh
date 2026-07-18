#!/usr/bin/env bash
# OOM guard: prover is the preferred kill target, capped at 24GB address space
echo 1000 > /proc/self/oom_score_adj 2>/dev/null || true
ulimit -v 25165824 2>/dev/null || true
# Pre-capture oracle outputs for every example input.
cd "$(dirname "$0")"
find examples -name "*.spthy" | while read -r f; do
  out="captures/$(echo "$f" | sed 's|/|__|g').out"
  [ -f "$out" ] || ./wf_oracle.sh "$f" > "$out" 2>&1
done
echo done
