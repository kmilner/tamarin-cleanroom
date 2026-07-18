#!/usr/bin/env bash
# R5 parse-side probe battery: for each path, print status | content-type | first bytes.
port="${1:-3131}"; shift
while IFS= read -r p; do
  [ -z "$p" ] && continue
  out=$(curl -s --path-as-is -w '\n%{http_code}|%{content_type}' "http://127.0.0.1:$port$p")
  code="${out##*$'\n'}"
  body="${out%$'\n'*}"
  printf '%-55s %s | %.110s\n' "$p" "$code" "$(echo "$body" | tr '\n' ' ')"
done
