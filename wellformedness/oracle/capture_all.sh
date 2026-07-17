#!/usr/bin/env bash
# Pre-capture oracle outputs for every example input.
cd "$(dirname "$0")"
find examples -name "*.spthy" | while read -r f; do
  out="captures/$(echo "$f" | sed 's|/|__|g').out"
  [ -f "$out" ] || ./wf_oracle.sh "$f" > "$out" 2>&1
done
echo done
