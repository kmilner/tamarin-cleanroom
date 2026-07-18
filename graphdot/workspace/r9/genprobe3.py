#!/usr/bin/env python3
# Round-9 probe battery #2: pin per-cell budgets to +-1 column.
# I: [Big(<16 $vars>)=87, Sib('a'*p+id)] with sib flat s stepping by 1 col.
# J: equal-pair trigger fine sweep (a,a), a = 41..46 (argfacts).
# K: triple trigger fine sweep (a,a,a), a = 28..32.
# L: mixed breakable/unbreakable pair at T ~= 84..90, both orders.
# M: 2-cell argfact pairs, second cell stepped by 1 col (budget of first).
import sys

def pvar(i):
    a = "abcdefghijklmnopqrstuvwxyz"
    return "$" + a[i//26] + a[i%26]

def pvars(n, off=0):
    return [pvar(i+off) for i in range(n)]

def bigfact(name, n, off=0):
    return f"{name}(<{', '.join(pvars(n, off))}>)"

def argfact(name, n, off=0):
    return f"{name}({', '.join(pvars(n, off))})"

def sib(p, idx, name="Sib"):
    return f"{name}('{'a'*p}{idx}')"

def in_prem(k, off=100):
    return f"In(<{', '.join(pvars(k, off))}>)"

rules, lemmas, combos = [], [], []
n = 0

def emit(name, prems, concls, comment):
    global n
    rules.append(
        f"// {comment}\n"
        f"rule {name}:\n  [ {', '.join(prems)} ]\n  --[ F{n}() ]->\n  [ {', '.join(concls)} ]\n")
    lemmas.append(f"lemma l_{name}:\n  exists-trace \"Ex #i. F{n}() @ #i\"\n")
    combos.append(name)
    n += 1

# --- Series I: Big 87 fixed; sib flat s = p+11, s stepping by 1 within windows
for s in [21,22,23,24, 26, 31,32,33,34, 36, 41,42,43,44, 46, 51,52,53,54,
          61,62,63,64, 71,72,73,74]:
    p = s - 11
    emit(f"I_{s:02d}", [in_prem(2)],
         [bigfact("Big", 16), sib(p, f"{n:02d}")],
         f"I: concl [Big 87, Sib {s}]")

# --- Series J: equal argfact pairs, T fine sweep around 87
# argfact flat = len(name)+5n+2. Use name lengths to hit odd widths.
# (name,nargs) -> flat: Faa+8 -> 45; Fa+8 -> 44; F+8 -> 43; Faaa+8 -> 46; F+8?
Jspecs = [("F", 8, 43), ("Fa", 8, 44), ("Faa", 8, 45), ("Faaa", 8, 46),
          ("Fh", 7, 39), ("Faaaa", 8, 47)]
for (nm, k, flat) in Jspecs:
    nm2 = "G" + nm[1:] if False else nm.replace("F", "H", 1)
    emit(f"J_{flat:02d}", [in_prem(2)],
         [argfact(nm, k), argfact(nm2 if nm2 != nm else nm+"x", k, off=k)],
         f"J: concl pair [{flat}, {flat}]")

# --- Series K: equal triples around T=87..96
Kspecs = [("F", 5, 28), ("Fa", 5, 29), ("Faa", 5, 30), ("Faaa", 5, 31), ("Faaaa", 5, 32)]
for (nm, k, flat) in Kspecs:
    emit(f"K_{flat:02d}", [in_prem(2)],
         [argfact(nm, k), argfact(nm.replace("F", "H", 1), k, off=k),
          argfact(nm.replace("F", "J", 1), k, off=2*k)],
         f"K: concl triple [{flat} x3]")

# --- Series L: breakable Faa(8)=45 with unbreakable Sib s, both orders, T 84..92
for s in [39, 41, 43, 45, 47]:
    p = s - 11
    emit(f"LA_{s:02d}", [in_prem(2)],
         [argfact("Faa", 8), sib(p, f"{n:02d}")],
         f"LA: concl [Faa 45, Sib {s}]")
    emit(f"LB_{s:02d}", [in_prem(2)],
         [sib(p, f"{n:02d}"), argfact("Faa", 8)],
         f"LB: concl [Sib {s}, Faa 45]")

# --- Series M: [Wide argfact 60, Nar argfact stepped by 1] -> budget of Wide
# Wide = Faa 11 args = 60. Nar: name len varies: F+4=23,Fa+4=24,Faa+4=25,Faaa+4=26,
# and 5-arg versions 28..31
Mspecs = [("F",4,23),("Fa",4,24),("Faa",4,25),("Faaa",4,26),
          ("F",5,28),("Fa",5,29),("Faa",5,30),("Faaa",5,31)]
for (nm, k, flat) in Mspecs:
    emit(f"M_{flat:02d}", [in_prem(2)],
         [argfact("Wide", 11), argfact(nm.replace("F","N",1), k, off=11)],
         f"M: concl [Wide 62, Nar {flat}]")

with open("probe3.spthy", "w") as f:
    f.write("theory BudgetPinProbe\nbegin\n\nbuiltins: hashing\n\n")
    for r in rules:
        f.write(r + "\n")
    for l in lemmas:
        f.write(l + "\n")
    f.write("end\n")
with open("probe3.names", "w") as f:
    for c in combos:
        f.write(c + "\n")
print(f"wrote probe3.spthy with {len(rules)} rules")
