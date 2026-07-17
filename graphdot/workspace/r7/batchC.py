#!/usr/bin/env python3
"""Batch C (trigger boundary): a single-atom target Ta('aaa'(L)) of flat p wraps
(its ')' peels, introducing \\l) iff p > effective_budget.  Sweep p by 1 to pin
the boundary p_fitmax = budget, for many sibling configs (2- and 3-cell)."""
import sys
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import build_theory, atom_fact

def ta(p):
    return atom_fact(p, "Ta")   # single-atom fact, flat p

rules = []; meta = []
def add(rn, concls, note):
    rules.append((rn, concls)); meta.append((rn, note))

def sweep2(q, ctr):
    sib = atom_fact(q, "Sib")
    for p in range(ctr-3, ctr+4):
        if p < 12: continue
        add(f"C2_q{q}_p{p}", [ta(p), sib], f"2cell q={q} p={p} expect~{ctr}")

# 2-cell: budget expected 87-q (Session 6).  Verify precisely across q.
for q in [10, 14, 20, 24, 28, 34, 40, 48, 58, 68, 78, 88, 98]:
    sweep2(q, 87 - q)

# 3-cell: [Ta(p), A(a), B(b)] ; expected budget 87-(a+b) (flat-sum).
def sweep3(a, b, ctr):
    A = atom_fact(a, "Ca"); B = atom_fact(b, "Cb")
    for p in range(ctr-3, ctr+4):
        if p < 12: continue
        add(f"C3_a{a}_b{b}_p{p}", [ta(p), A, B], f"3cell a={a} b={b} p={p} expect~{ctr}")

for (a, b) in [(25, 14), (40, 40), (10, 10), (50, 20), (30, 30), (14, 68), (48, 48)]:
    sweep3(a, b, 87 - (a + b))

thy = build_theory("R7C", rules)
open(sys.argv[1], "w").write(thy)
open(sys.argv[1] + ".meta", "w").write("\n".join(f"{n}\t{d}" for n, d in meta))
print("rules:", len(rules))
