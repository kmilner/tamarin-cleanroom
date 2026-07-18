#!/usr/bin/env python3
# Generate a probe theory: 2-cell conclusion groups [Big(K distinct 2-char vars),
# Sib(single public constant of controlled length)] to measure how a group's
# cells share the row. Distinct vars / occ-1 constants => no abbreviation.
import sys, itertools

# distinct 2-char var names: aa, ab, ... (avoid clashing with builtins)
def varname(i):
    a = "abcdefghijklmnopqrstuvwxyz"
    return a[i//26] + a[i%26]

def big(K):
    elems = ", ".join(varname(i) for i in range(K))
    return f"Big(<{elems}>)"

# single public constant of P a-chars => Sib flat = len("Sib( ") + (P+2) + len(" )")
def sib(P, idx):
    return f"Sib('{'a'*P}{idx}')"   # trailing idx keeps constants distinct (occ 1)

rules = []
lemmas = []
# sweep: K in a few sizes, sib length P in a range
combos = []
n = 0
for K in [8, 10, 12, 16, 20]:
    for P in [1, 3, 5, 8, 10, 12, 15, 18, 20, 25, 30, 40, 50, 60]:
        name = f"R_{K}_{P}"
        # unique fresh so the rule fires; premise In feeds the vars
        invars = ", ".join(varname(i) for i in range(K))
        rules.append(
            f"rule {name}:\n"
            f"  [ Fr(~n{n}), In(<{invars}>) ]\n"
            f"  --[ Fire{n}() ]->\n"
            f"  [ {big(K)}, {sib(P, n)} ]\n"
        )
        lemmas.append(f"lemma l_{name}:\n  exists-trace \"Ex #i. Fire{n}() @ #i\"\n")
        combos.append((name, K, P))
        n += 1

with open("probe.spthy", "w") as f:
    f.write("theory GroupShareProbe\nbegin\n\nbuiltins: hashing\n\n")
    for r in rules: f.write(r + "\n")
    for l in lemmas: f.write(l + "\n")
    f.write("end\n")

with open("probe.combos", "w") as f:
    for name, K, P in combos:
        f.write(f"{name}\t{K}\t{P}\n")

print(f"wrote probe.spthy with {len(rules)} rules")
