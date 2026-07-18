#!/usr/bin/env python3
# Round-9 probe battery: cross-row coupling of record-row wrap budgets.
# All cell terms use short public $-vars (never abbreviated) except Sib
# constants which are occ-1 (never abbreviated).
import sys

def pvar(i):
    a = "abcdefghijklmnopqrstuvwxyz"
    return "$" + a[i//26] + a[i%26]

def pvars(n, off=0):
    return [pvar(i+off) for i in range(n)]

def tuple_term(elems):
    return "<" + ", ".join(elems) + ">"

# Big tuple fact of n $-vars: flat = len(name)+2 + (5n-2+2) + 2 = len(name)+5n+4
def bigfact(name, n, off=0):
    return f"{name}({tuple_term(pvars(n, off))})"

# multi-arg fact of n $-vars: flat = len(name)+2 + (5n-2) + 2 = len(name)+5n+2
def argfact(name, n, off=0):
    return f"{name}({', '.join(pvars(n, off))})"

# Sib with occ-1 constant: flat = len("Sib")+2 + (p+2+len(idx)) + 2
def sib(p, idx, name="Sib"):
    return f"{name}('{'a'*p}{idx}')"

# In premise from a tuple of k $-vars: flat = 2+2 + (5k) + 2 = 5k+7  (name "In")
def in_prem(k, off=40):
    return f"In({tuple_term(pvars(k, off))})"

rules = []
lemmas = []
combos = []
n = 0

def emit(name, prems, concls, comment):
    global n
    rules.append(
        f"// {comment}\n"
        f"rule {name}:\n"
        f"  [ {', '.join(prems)} ]\n"
        f"  --[ F{n}() ]->\n"
        f"  [ {', '.join(concls)} ]\n"
    )
    lemmas.append(f"lemma l_{name}:\n  exists-trace \"Ex #i. F{n}() @ #i\"\n")
    combos.append(name)
    n += 1

# --- Series A: concl [Big 87, Sib 33]; sweep premise-row width (5k+7)
# Big: "Big" + 16 elems -> 3+16*5+4 = 87.  Sib: p=22, idx 2ch -> 5+26+2 = 33
for k in [2, 4, 6, 8, 10, 12, 14, 16, 18, 22]:
    emit(f"A_{5*k+7:03d}", [in_prem(k)],
         [bigfact("Big", 16), sib(22, f"{n:02d}")],
         f"A: prem width {5*k+7}, concl [87, 33]")

# --- Series B: order swap [Sib 33, Big 87]; prem width sweep
for k in [4, 8, 12, 16, 22]:
    emit(f"B_{5*k+7:03d}", [in_prem(k)],
         [sib(22, f"{n:02d}"), bigfact("Big", 16)],
         f"B: prem width {5*k+7}, concl [33, 87]")

# --- Series C: lone concl cell Big 92 ("Bigg"+17: 4+85+4=93); prem sweep
for k in [2, 8, 16, 24]:
    emit(f"C_{5*k+7:03d}", [in_prem(k)],
         [bigfact("Bigg", 17)],
         f"C: prem width {5*k+7}, lone concl 93")

# --- Series E: info-row width via rule-name length; prem 17, concl [87, 33]
for pad in [0, 30, 60, 110]:
    nm = "E" + "x" * pad
    emit(f"{nm}_{pad:03d}"[:60] if False else nm + f"_{pad:03d}",
         [in_prem(2)],
         [bigfact("Big", 16), sib(22, f"{n:02d}")],
         f"E: info padded by rule-name len {pad}, concl [87, 33]")

# --- Series D: pair grid under WIDE prem (127); argfacts of $-vars
# argfact width = len(name)+5n+2; name len 3 -> 5n+5
pairs = [(5,5), (7,7), (8,8), (9,7), (7,9), (11,5), (5,11), (9,9), (10,8)]
for (a, b) in pairs:
    emit(f"D_{5*a+5:02d}_{5*b+5:02d}", [in_prem(24)],
         [argfact("Faa", a), argfact("Fbb", b, off=a)],
         f"D: prem 127, concl [{5*a+5}, {5*b+5}]")

# --- Series G: same pair grid under NARROW prem (17)
for (a, b) in pairs:
    emit(f"G_{5*a+5:02d}_{5*b+5:02d}", [in_prem(2)],
         [argfact("Faa", a), argfact("Fbb", b, off=a)],
         f"G: prem 17, concl [{5*a+5}, {5*b+5}]")

# --- Series F: 3-cell concl; prem wide vs narrow
trip = [(5,5,5), (8,4,4), (4,4,8), (6,6,6), (8,8,8)]
for (a, b, c) in trip:
    emit(f"FW_{a}_{b}_{c}", [in_prem(24)],
         [argfact("Faa", a), argfact("Fbb", b, off=a), argfact("Fcc", c, off=a+b)],
         f"FW: prem 127, concl [{5*a+5}, {5*b+5}, {5*c+5}]")
    emit(f"FN_{a}_{b}_{c}", [in_prem(2)],
         [argfact("Faa", a), argfact("Fbb", b, off=a), argfact("Fcc", c, off=a+b)],
         f"FN: prem 17, concl [{5*a+5}, {5*b+5}, {5*c+5}]")

# --- Series H: replicate (Big 87, sib s) with wide prem 127
for p in [10, 20, 30, 40, 55]:
    emit(f"H_{p:02d}", [in_prem(24)],
         [bigfact("Big", 16), sib(p, f"{n:02d}")],
         f"H: prem 127, concl [87, {p+11}]")

with open("probe2.spthy", "w") as f:
    f.write("theory RowCouplingProbe\nbegin\n\nbuiltins: hashing\n\n")
    for r in rules:
        f.write(r + "\n")
    for l in lemmas:
        f.write(l + "\n")
    f.write("end\n")

with open("probe2.names", "w") as f:
    for c in combos:
        f.write(c + "\n")

print(f"wrote probe2.spthy with {len(rules)} rules")
