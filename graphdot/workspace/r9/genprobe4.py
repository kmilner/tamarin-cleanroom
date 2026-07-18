#!/usr/bin/env python3
# Round-9 probe battery #3: pin occupancy C for more shapes via the
# stepped-narrow-sibling method: [Shape-cell, Nar stepped by 1] — the Nar
# fit/wrap transition pins C(Shape) exactly (Nar wraps iff nar > 87 - C).
import sys

def pvar(i):
    a = "abcdefghijklmnopqrstuvwxyz"
    return "$" + a[i//26] + a[i%26]

def pvars(n, off=0):
    return [pvar(i+off) for i in range(n)]

def argfact(name, n, off=0):
    return f"{name}({', '.join(pvars(n, off))})"

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

def in_prem(k, off=100):
    return f"In(<{', '.join(pvars(k, off))}>)"

# Nar cell: name len varies to step flat by 1: N/Na/Naa/Naaa + 4 or 5 args
NAR = [("N",4,23),("Na",4,24),("Naa",4,25),("Naaa",4,26),
       ("N",5,28),("Na",5,29),("Naa",5,30),("Naaa",5,31)]

def nar(spec, off):
    nm, k, flat = spec
    return argfact(nm.replace("N","N",1), k, off), flat

# Shape A: func node h(x) inside an argfact:
# "Fh( $aa, $ab, $ac, $ad, $ae, $af, $ag, h($ah) )" flat = 4+7*5+2+9-... compute in name
shapeA = "Fnn(" + ", ".join(pvars(7)) + ", h($ah))"   # display "Fnn( $aa, ..., h($ah) )"
# flat = 5 + 7*5 + 2 + len("h($ah)") + 2 = 5+35+2+6+2? -> compute: args join
flatA = len("Fnn( ") + len(", ".join(pvars(7) + ["h($ah)"])) + len(" )")

# Shape B: two quoted consts in a multi-arg fact
shapeB = "Fqq('aaaa', 'bbbb', " + ", ".join(pvars(5, 8)) + ")"
flatB = len("Fqq( ") + len(", ".join(["'aaaa'", "'bbbb'"] + pvars(5, 8))) + len(" )")

# Shape C: pair tuple arg (n=2 -> dtop 0 expected)
shapeC = "Fpp(<" + ", ".join(pvars(2, 16)) + ">, " + ", ".join(pvars(5, 18)) + ")"
flatC = len("Fpp( ") + len("<$aq, $ar>, ") + len(", ".join(pvars(5, 18))) + len(" )")

# Shape D: 4-elem tuple arg (dtop 4 expected)
shapeD = "Fdd(<" + ", ".join(pvars(4, 16)) + ">, " + ", ".join(pvars(3, 20)) + ")"
flatD = len("Fdd( ") + len("<" + ", ".join(pvars(4, 16)) + ">, ") + len(", ".join(pvars(3, 20))) + len(" )")

# Shape E: fresh var in fact
shapeE = "Fee(~nn, " + ", ".join(pvars(6, 16)) + ")"
flatE = len("Fee( ") + len("~nn, ") + len(", ".join(pvars(6, 16))) + len(" )")

for tag, shape, flat, prem_extra in [
    ("A", shapeA, flatA, []),
    ("B", shapeB, flatB, []),
    ("C", shapeC, flatC, []),
    ("D", shapeD, flatD, []),
    ("E", shapeE, flatE, ["Fr(~nn)"]),
]:
    for spec in NAR:
        nnm, k, nflat = spec
        cell, _ = nar(spec, 30)
        emit(f"P{tag}_{flat:02d}_{nflat:02d}", prem_extra + [in_prem(2)],
             [shape, cell],
             f"P{tag}: [{tag}-shape {flat}, Nar {nflat}]")

# Fill-saturation probes: [Big(<16 $vars>)=87, Sib atom s] for very large s
def sib(p, idx):
    return f"Sib('{'a'*p}{idx}')"
def bigfact(name, nn, off=0):
    return f"{name}(<{', '.join(pvars(nn, off))}>)"
for s in [78, 84, 92, 104, 120]:
    emit(f"Q_{s:03d}", [in_prem(2)], [bigfact("Big", 16), sib(s-11, f"{n:02d}")],
         f"Q: [Big87, Sib {s}] fill saturation")

with open("probe4.spthy", "w") as f:
    f.write("theory ShapePinProbe\nbegin\n\nbuiltins: hashing\n\n")
    for r in rules:
        f.write(r + "\n")
    for l in lemmas:
        f.write(l + "\n")
    f.write("end\n")
with open("probe4.names", "w") as f:
    for c in combos:
        f.write(c + "\n")
print(f"wrote probe4.spthy with {len(rules)} rules; flats A={flatA} B={flatB} C={flatC} D={flatD} E={flatE}")
