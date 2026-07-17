#!/usr/bin/env bash
# Byte-parity harness for Unit E (macro expansion).
#
# For a (macro-theory, hand-expanded-equivalent) pair, run the black-box oracle
# on both and compare the ONLY thing macro expansion is responsible for: the
# oracle's *expanded* rendering.
#
# Observed invariant (see BEHAVIOR.md): a macro theory's closed output differs
# from its hand-expanded equivalent ONLY in cosmetic scaffolding --
#   (1) the preserved `macros:` declaration block,
#   (2) the primary `rule (modulo E)` / restriction / lemma line still showing
#       the un-expanded macro *call*,
#   (3) the explicit `/* rule (modulo AC) ... */` variant block that a macro
#       forces (vs `/* has exactly the trivial AC variant */` when no macro).
# The EXPANDED content the oracle computes for the macro theory (shown in the
# modulo-AC variant / `expanded formula` / `guarded formula`) is byte-identical
# to the primary rendering of the hand-expanded equivalent.
#
# This script does a canonicalising comparison: it strips scaffolding (1)-(3)
# and the un-expanded call lines, leaving the expanded semantic core, then
# diffs. Exit 0 == byte-parity of expanded content.
#
# Usage: byteparity.sh <macro.spthy> <expanded.spthy>
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ORACLE="$HERE/../oracle/hs_oracle.sh"

strip_volatile() {
  grep -vE 'maude tool:|checking version:|checking installation:|Compiled at:|Git revision:|Tamarin version|Maude version|processing time|analyzed:|Generated from:'
}

# Reduce an oracle output to its expanded semantic core:
#  - drop the macros: declaration block (starts "macros:", continues on lines
#    that are indented continuations ending in ',' or being the last macro).
#  - drop the primary "rule (modulo E)" header+body when an explicit modulo-AC
#    variant follows (keep the AC variant, un-commented).
# Rather than a fragile structural rewrite, we normalise textually:
#  * delete lines belonging to the macros: block
#  * delete "rule (modulo E)" ... up to the "/*" that opens its AC variant,
#    then un-comment the "rule (modulo AC)" block.
# For robustness across theories we use a python filter.
python3 - "$1" "$2" "$ORACLE" <<'PY'
import subprocess, sys, re
macro_f, exp_f, oracle = sys.argv[1], sys.argv[2], sys.argv[3]

def run(f):
    out = subprocess.run(["bash", oracle, f], capture_output=True, text=True,
                          env={"ORACLE_TIMEOUT":"90","PATH":"/home/linuxbrew/.linuxbrew/bin:/usr/bin:/bin"}).stdout
    return out.splitlines()

VOL = re.compile(r'maude tool:|checking version:|checking installation:|Compiled at:|Git revision:|Tamarin version|Maude version|processing time|analyzed:|Generated from:')

def core(lines):
    """Reduce to expanded semantic core: drop macros block, drop primary
    modulo-E rule when an AC variant exists, un-comment AC variant, drop the
    'expanded formula'/'guarded formula' comment wrappers (keep their content),
    keep everything else. Collapse blank lines."""
    out = []
    i = 0
    n = len(lines)
    while i < n:
        ln = lines[i]
        if VOL.search(ln):
            i += 1; continue
        s = ln.strip()
        # macros: block -- header then continuation lines until one w/o trailing ','
        if s.startswith('macros:'):
            # consume this line and continuation lines (indented) while they end with ','
            while i < n and lines[i].rstrip().endswith(','):
                i += 1
            i += 1  # the last macro line (no trailing comma)
            continue
        # primary modulo E rule with a following AC variant: drop E, keep AC
        if s.startswith('rule (modulo E)'):
            # look ahead: gather the E-rule block until blank line
            j = i
            block = []
            while j < n and lines[j].strip() != '':
                block.append(lines[j]); j += 1
            # is the next non-blank an AC variant comment?
            k = j
            while k < n and lines[k].strip() == '':
                k += 1
            if k < n and lines[k].strip() == '/*' and k+1 < n and lines[k+1].strip().startswith('rule (modulo AC)'):
                # skip E block entirely; emit un-commented AC block
                m = k+1
                acblock = []
                while m < n and lines[m].strip() != '*/':
                    acblock.append(lines[m]); m += 1
                # acblock may contain "variants (modulo AC)" tail -> keep as-is
                out.extend(x for x in acblock)
                i = m+1
                continue
            else:
                # no AC variant (trivial) -> keep E block but relabel to AC-agnostic
                out.extend(block)
                i = j
                continue
        # 'expanded formula:' / 'guarded formula...' comment wrappers: keep inner
        if s == 'expanded formula:' or s.startswith('guarded formula'):
            i += 1
            continue
        if s in ('/*','*/'):
            i += 1; continue
        out.append(ln.rstrip())
        i += 1
    # Final normalisation so the *expanded term content* compares equal
    # regardless of the remaining cosmetic scaffolding (rule header label
    # "modulo E" vs "modulo AC", 3- vs 5-space indentation, and the
    # "trivial AC variant" note that only the no-macro side emits).
    RULEHDR = re.compile(r'^rule \(modulo (?:E|AC)\) ')
    res = []
    for ln in out:
        s = ln.strip()
        if s == '/* has exactly the trivial AC variant */':
            continue
        s = RULEHDR.sub('rule ', s)     # unify header label
        if s == '':
            continue
        res.append(s)                   # lstrip/rstrip (indentation-agnostic)
    return res

cm = core(run(macro_f))
ce = core(run(exp_f))
if cm == ce:
    print("BYTE-PARITY OK: expanded content identical (%d lines)" % len(cm))
    sys.exit(0)
else:
    print("BYTE-PARITY MISMATCH")
    import difflib
    for d in difflib.unified_diff(cm, ce, "macro-core", "expanded-core", lineterm=""):
        print(d)
    sys.exit(1)
PY
