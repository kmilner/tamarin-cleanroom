#!/usr/bin/env python3
"""Batch G: does cell ORDER change the wrap boundary at the margin?
A = single-atom fact flat p (fill 'a'), B = single-atom flat q (fill 'b').
Compare A's wrap boundary in [A,B] vs [B,A].  Also 3-cell permutations, and a
multi-arg sibling.  Distinct fills -> no abbreviation."""
import sys
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import build_theory, atom_fact

rules=[]; meta=[]
def add(rn, concls, note): rules.append((rn,concls)); meta.append((rn,note))

# G1: 2-cell order, A near boundary, B fixed flat 40 -> A boundary should be 47.
for q in [40, 60]:
    B = atom_fact(q, "Bb", fill='b')
    for p in range(87-q-4, 87-q+5):
        if p < 12: continue
        A = atom_fact(p, "Aa", fill='a')
        add(f"G1_q{q}_AB_p{p}", [A, B], f"[A,B] q={q} p={p}")
        add(f"G1_q{q}_BA_p{p}", [B, A], f"[B,A] q={q} p={p}")

# G2: multi-arg sibling M (a 5-arg fact of flat ~40) vs A near boundary.
def marg(nargs, w, name, fill):
    # NAME( 'x..','x..',... ) with nargs atoms each of width w (incl quotes)
    atoms = ", ".join("'"+fill*(w-2)+chr(ord('0')+i)+"'" for i in range(nargs))
    return f"{name}({atoms})"
# skip G2 (fill collisions tricky) -- rely on G1/G3

# G3: 3-cell order permutations of [A(p), B(30), C(30)] ; find A boundary per position.
import itertools
for perm in [("A","B","C"), ("B","A","C"), ("B","C","A")]:
    for p in range(20, 30):
        A=atom_fact(p,"Aa",fill='a'); B=atom_fact(30,"Bb",fill='b'); C=atom_fact(30,"Cc",fill='c')
        cells=[{"A":A,"B":B,"C":C}[k] for k in perm]
        add(f"G3_{''.join(perm)}_p{p}", cells, f"perm {perm} p={p}")

thy=build_theory("R7G", rules)
open(sys.argv[1],"w").write(thy)
open(sys.argv[1]+".meta","w").write("\n".join(f"{n}\t{d}" for n,d in meta))
print("rules:", len(rules))
