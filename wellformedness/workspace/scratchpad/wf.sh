#!/usr/bin/env bash
# Run oracle, print ONLY the wellformedness block (success line or WARNING /* */).
ORACLE=/home/kamilner/tamarin-cleanroom/wellformedness/oracle/wf_oracle.sh
out="$("$ORACLE" "$@" 2>&1)"
if printf '%s\n' "$out" | grep -qF 'All wellformedness checks were successful.'; then
  echo '/* All wellformedness checks were successful. */'
  exit 0
fi
# Extract the /* ... */ block that contains the WARNING header.
printf '%s\n' "$out" | awk '
  /^\/\*$/ { buf=$0"\n"; inblk=1; haswarn=0; next }
  inblk==1 {
    buf=buf $0 "\n";
    if ($0 ~ /WARNING: the following wellformedness/) haswarn=1;
    if ($0 ~ /^\*\/$/) { if (haswarn) printf "%s", buf; inblk=0 }
    next
  }
'
