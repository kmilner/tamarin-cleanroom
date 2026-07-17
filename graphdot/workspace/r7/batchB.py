#!/usr/bin/env python3
"""Batch B (fine): pin the effective FILL budget B exactly, per sibling flat q.
Target = Tgt(<'aaa'(W), 'y'>); 'y' stays on line0 iff B >= W+13, peels iff
B <= W+12.  So B = (max W with 'y' on line0) + 13.  Sweep W by 1 across the
transition, for several q (single-atom sibling), both orders."""
import sys
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import build_theory, atom_fact

def tgt(w):
    return f"Tgt(<'{'a'*w}', 'y'>)"

rules = []; meta = []
def add(rn, concls, note):
    rules.append((rn, concls)); meta.append((rn, note))

# (q, W-center) ; sweep W in [c-7, c+7]
CONF = [(0, 74), (10, 72), (20, 65), (34, 64), (48, 58), (68, 51), (88, 51), (98, 51)]
for q, c in CONF:
    for w in range(c-7, c+8):
        if q == 0:
            add(f"BL_w{w}", [tgt(w)], f"alone W={w}")
        else:
            sib = atom_fact(q, "Sib")
            add(f"BF_q{q}_w{w}", [tgt(w), sib], f"[TGT,SIB] q={q} W={w}")

thy = build_theory("R7B", rules)
open(sys.argv[1], "w").write(thy)
open(sys.argv[1] + ".meta", "w").write("\n".join(f"{n}\t{d}" for n, d in meta))
print("rules:", len(rules))
