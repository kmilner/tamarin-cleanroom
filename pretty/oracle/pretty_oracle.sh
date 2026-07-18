#!/usr/bin/env bash
# Black-box oracle for the THEORY PRETTY-PRINTER clean-room.
#
# Runs the sanctioned Haskell tamarin-prover on a .spthy file WITHOUT --prove
# (the theory echo prints at theory-load; no proof search) and, by default,
# prints the EXTRACTED THEORY-ECHO BLOCK — the exact `theory <name> begin …
# end` text your reimplementation must reproduce byte-for-byte.
#
#   pretty_oracle.sh <file.spthy> [extra args]   # -> extracted theory echo
#   RAW=1 pretty_oracle.sh <file.spthy> [args]   # -> full HS output (2>&1),
#                                                   #    for parse errors / the
#                                                   #    wf block you are NOT
#                                                   #    reimplementing.
#
# The extracted block DROPS the trailing wellformedness comment (a separate
# slice) and the volatile `/* Generated from: … */` build stamp, and everything
# after `end` (the summary-of-summaries).  Interior comments — rule AC-variant
# blocks and `guarded formula characterizing …` — are KEPT; they are your
# output.  This is identical to the extraction the acceptance gate applies to
# both sides, so `diff <(pretty_oracle.sh f) <(your_binary f | extract)` == the
# gate's verdict.
echo 1000 > /proc/self/oom_score_adj 2>/dev/null || true
ulimit -v 25165824 2>/dev/null || true
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
HS="${HS_PATH:-/home/kamilner/tamarin-rs/tamarin-prover-testing/.stack-work/install/x86_64-linux-tinfo6/ec0cb11b1bfcf8776d45e0357bbc6d6ff2077f9222735af22115429c8cdfcef1/9.6.7/bin/tamarin-prover}"
f="$1"; shift
DERIVCHECK_TIMEOUT="${DERIVCHECK_TIMEOUT:-10}"

if [ -n "${RAW:-}" ]; then
    exec timeout "${ORACLE_TIMEOUT:-60}" "$HS" --derivcheck-timeout="$DERIVCHECK_TIMEOUT" "$f" "$@" 2>&1
fi

timeout "${ORACLE_TIMEOUT:-60}" "$HS" --derivcheck-timeout="$DERIVCHECK_TIMEOUT" "$f" "$@" 2>/dev/null \
| grep -v -e '^Git revision:' -e '^Compiled at:' -e '^[[:space:]]*processing time:' -e '^[[:space:]]*analyzed:' \
| awk '
    /^theory /              { cap=1 }
    !cap                    { next }
    /^\/\* All wellformedness checks were successful\. \*\/$/ { next }
    /^\/\*$/ {
        if ((getline nxt) > 0) {
            if (nxt == "WARNING: the following wellformedness checks failed!" || nxt == "Generated from:") {
                while ((getline z) > 0) { if (z == "*/") break }
                next
            }
            print; print nxt; next
        }
        print; next
    }
    { print }
    /^end$/                 { cap=0 }
'
