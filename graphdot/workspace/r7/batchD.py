#!/usr/bin/env python3
"""Batch D: confirm the coupled 'occupied width' model.
budget_i = max(87 - sum_{j!=i} occ_j, 20), occ_j = flat_j if j fits else j's
wrapped MAX physical-line width.  For a single-atom fact of flat f, occ = f-2.

D2: [Ta(p), A(a single atom), B(10)] ; A wraps at Ta's boundary -> budget=79-a
    (flat-sum would give 77-a).  Sweep p to pin budget, for several a.
D3: [Ta(p), A(a), A2(a)] two equal wrapping siblings -> measure deviation.
D4: read occ of a forced-wrapped single atom directly (its widest line)."""
import sys
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import build_theory, atom_fact

def ta(p): return atom_fact(p, "Ta")

rules = []; meta = []
def add(rn, concls, note):
    rules.append((rn, concls)); meta.append((rn, note))

# D2: one wrapping sibling A(a) + a small non-wrapping B(10)
for a in [24, 30, 40, 50]:
    A = atom_fact(a, "Aa"); B = atom_fact(10, "Bb")
    ctr = 79 - a  # model prediction
    for p in range(ctr-4, ctr+4):
        if p < 12: continue
        add(f"D2_a{a}_p{p}", [ta(p), A, B], f"D2 a={a} p={p} predict~{ctr}")

# D3: two equal wrapping siblings A(a), A2(a)
for a in [24, 30, 40]:
    A = atom_fact(a, "Aa"); A2 = atom_fact(a, "Ac")
    ctr = 87 - 2*(a-2)  # model: budget = 87 - 2*occ = 87-2(a-2)
    for p in range(ctr-4, ctr+4):
        if p < 12: continue
        add(f"D3_a{a}_p{p}", [ta(p), A, A2], f"D3 a={a} p={p} predict~{ctr}")

# D4: force a single atom to wrap (huge partner) and read its widest line.
# Wp(f) forced to wrap by Huge; also vary f.
for f in [24, 30, 40, 50, 60]:
    Wp = atom_fact(f, "Wp"); Huge = atom_fact(90, "Hg")
    add(f"D4_f{f}", [Wp, Huge], f"D4 read occ of Wp flat {f} (wraps)")

thy = build_theory("R7D", rules)
open(sys.argv[1], "w").write(thy)
open(sys.argv[1] + ".meta", "w").write("\n".join(f"{n}\t{d}" for n, d in meta))
print("rules:", len(rules))
