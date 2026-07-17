#!/usr/bin/env bash
# Formula-context parity harness for Unit E (macro expansion).
#
# For contexts where macro expansion surfaces as a *processed formula* rather
# than an AC rule variant -- lemmas, restrictions, accountability lemmas, and
# case tests -- the oracle prints the un-expanded macro CALL in the primary
# rendering and the fully EXPANDED term in a `guarded formula ...` (or
# `expanded formula:`) comment block. Macro expansion is responsible for that
# expanded content.
#
# This harness runs the black-box oracle (full env, so Unicode logic output
# encodes correctly) on a (macro-theory, hand-inlined-equivalent) pair and
# compares ONLY the expanded/guarded-formula content. Both theories compute the
# same expanded content, so the blocks must be byte-identical. Exit 0 == parity.
#
# Usage: formula_parity.sh <macro.spthy> <expanded.spthy>
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ORACLE="$HERE/../oracle/hs_oracle.sh"

python3 - "$1" "$2" "$ORACLE" <<'PY'
import subprocess, sys, re
macro_f, exp_f, oracle = sys.argv[1], sys.argv[2], sys.argv[3]

def run(f):
    return subprocess.run(
        ["bash", oracle, f], capture_output=True, text=True,
        env={"ORACLE_TIMEOUT": "120", "PATH": "/home/linuxbrew/.linuxbrew/bin:/usr/bin:/bin",
             "LC_ALL": "C.UTF-8", "LANG": "C.UTF-8"},
    ).stdout.splitlines()

HDR = re.compile(r'guarded formula characterizing|^\s*expanded formula:')

def blocks(lines):
    """Concatenate every guarded/expanded-formula block: the lines from just
    after such a header up to the closing '*/'. Whitespace-normalised."""
    out, i, n = [], 0, len(lines)
    while i < n:
        if HDR.search(lines[i]):
            i += 1
            while i < n and lines[i].strip() != '*/':
                s = lines[i].strip()
                if s:
                    out.append(re.sub(r'\s+', ' ', s))
                i += 1
        else:
            i += 1
    return out

bm = blocks(run(macro_f))
be = blocks(run(exp_f))
if not bm:
    print("NO EXPANDED/GUARDED BLOCKS FOUND (oracle produced none)")
    sys.exit(2)
if bm == be:
    print("FORMULA-PARITY OK: expanded content identical (%d block-lines)" % len(bm))
    sys.exit(0)
print("FORMULA-PARITY MISMATCH")
import difflib
for d in difflib.unified_diff(bm, be, "macro-guarded", "expanded-guarded", lineterm=""):
    print(d)
sys.exit(1)
PY
