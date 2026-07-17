#!/usr/bin/env python3
"""Batch A (coarse): map the effective FILL budget B of a wrapping tuple cell as
a function of ONE sibling's flat width q, in both group orders, incl. wrapping
siblings.  One rule per config; read line-0 element count M of the tuple
(5-char elems 'eNN' at open_col 6, sep 2): B in [7M+4, 7M+10]."""
import sys
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import build_theory, atom_fact

def bigN(n, name="Big"):
    return f"{name}(" + "<" + ", ".join(f"'e{i:02d}'" for i in range(1, n+1)) + ">)"

rules = []; meta = []
def add(rn, concls, note):
    rules.append((rn, concls)); meta.append((rn, note))

N = 30  # plenty of elements to overflow any budget
SIBS = [10, 12, 14, 16, 18, 20, 22, 24, 28, 34, 40, 48, 58, 68, 78, 88, 98]
for q in SIBS:
    sib = atom_fact(q, "Sib")
    add(f"AF_q{q}", [bigN(N), sib], f"[BIG,SIB] q={q}")   # target first
    add(f"AS_q{q}", [sib, bigN(N)], f"[SIB,BIG] q={q}")   # sibling first (order test)

# lone target (no sibling) => budget should be 87
add("ALONE", [bigN(N)], "lone target, expect budget 87")

# Exact Wide-datum reproduction: 3-cell [big, 25, 14] in three orders.
add("WIDE_big25_14", [bigN(10), atom_fact(25, "Ack"), atom_fact(14, "Out")], "wide [big,25,14]")
add("WIDE_25_big_14", [atom_fact(25, "Ack"), bigN(10), atom_fact(14, "Out")], "wide [25,big,14]")
add("WIDE_14_25_big", [atom_fact(14, "Out"), atom_fact(25, "Ack"), bigN(10)], "wide [14,25,big]")

thy = build_theory("R7A", rules)
open(sys.argv[1], "w").write(thy)
open(sys.argv[1] + ".meta", "w").write("\n".join(f"{n}\t{d}" for n, d in meta))
print("rules:", len(rules))
