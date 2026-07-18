#!/usr/bin/env bash
# OOM guard: prover is the preferred kill target, capped at 24GB address space
echo 1000 > /proc/self/oom_score_adj 2>/dev/null || true
ulimit -v 25165824 2>/dev/null || true
# Black-box oracle: prints the HS tamarin-prover's full output (incl. the
# wellformedness WARNING section). Usage: wf_oracle.sh <file.spthy> [extra args e.g. --diff]
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
f="$1"; shift
exec timeout ${ORACLE_TIMEOUT:-60} "/home/kamilner/tamarin-rs/tamarin-prover-testing/.stack-work/install/x86_64-linux-tinfo6/ec0cb11b1bfcf8776d45e0357bbc6d6ff2077f9222735af22115429c8cdfcef1/9.6.7/bin/tamarin-prover" "$f" "$@" 2>&1
