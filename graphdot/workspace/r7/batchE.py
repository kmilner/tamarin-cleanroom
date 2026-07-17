#!/usr/bin/env python3
"""Batch E (clean, distinct fills): confirm budget_i = max(87 - sum occ_j, 20),
occ_j = flat_j if j fits else wrapped max-line width (= flat-2 for single atoms).
Every fact uses a DISTINCT fill char so equal-length atoms never collide -> no
abbreviation.  Target Ta uses fill 'z'."""
import sys
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import build_theory, atom_fact

def ta(p): return atom_fact(p, "Ta", fill='z')

rules = []; meta = []
def add(rn, concls, note):
    rules.append((rn, concls)); meta.append((rn, note))

# 3-cell trigger boundary sweeps, distinct fills 'c','d' for the two siblings.
def sweep3(a, b, ctr, tag):
    A = atom_fact(a, "Ca", fill='c'); B = atom_fact(b, "Cb", fill='d')
    for p in range(ctr-4, ctr+5):
        if p < 12: continue
        add(f"{tag}_p{p}", [ta(p), A, B], f"3cell a={a} b={b} p={p} predict~{ctr}")

# occ-model predicted centers (assume both siblings wrap where relevant):
# budget = max(87 - occ_a - occ_b, 20); occ=flat-2 if that sibling wraps at bdry.
sweep3(30, 30, 31, "E_30_30")    # both wrap -> 87-28-28=31 ; flatsum(floor)=27
sweep3(24, 24, 43, "E_24_24")    # 87-22-22=43 ; flatsum=39
sweep3(40, 40, 20, "E_40_40")    # 87-38-38<20 -> 20 ; flatsum=20
sweep3(50, 20, 21, "E_50_20")    # 87-48-18=21 (or floor) ; flatsum=20
sweep3(40, 10, 39, "E_40_10")    # A(40) wraps? B(10) fits. 87-38-10=39 ; flatsum=37
sweep3(60, 10, 27, "E_60_10")    # 87-58-10=19->? ; check floor vs occ
sweep3(25, 14, 48, "E_25_14")    # neither wraps -> 87-39=48 (re-verify clean)

# Floor / single huge sibling (2-cell): budget = max(87-occ, 20).
for q in [68, 78, 88, 98]:
    S = atom_fact(q, "Hg", fill='h')
    ctr = max(87 - (q-2), 20)
    for p in range(ctr-4, ctr+5):
        if p < 12: continue
        add(f"E_hg{q}_p{p}", [ta(p), S], f"2cell huge q={q} p={p} predict~{ctr}")

# ORDER test with WRAPPING siblings: [A,B,Ta] and [Ta,A,B] and [A,Ta,B], a=b=30.
for order, tag in [(["A","B","T"], "ord_ABT"), (["T","A","B"], "ord_TAB"), (["A","T","B"], "ord_ATB")]:
    A = atom_fact(30, "Ca", fill='c'); B = atom_fact(30, "Cb", fill='d')
    for p in [29, 30, 31, 32]:
        cells = []
        for k in order:
            cells.append({"A": A, "B": B, "T": ta(p)}[k])
        add(f"E_{tag}_p{p}", cells, f"order {tag} p={p}")

thy = build_theory("R7E", rules)
open(sys.argv[1], "w").write(thy)
open(sys.argv[1] + ".meta", "w").write("\n".join(f"{n}\t{d}" for n, d in meta))
print("rules:", len(rules))
