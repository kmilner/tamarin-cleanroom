#!/usr/bin/env python3
# Round-10 battery E: pin the quote-in-tuple anomaly (QC) and the union
# occupancy/bonus, with pure-var controls at the same [Faa 45, sib] base.
#   PF: control — pure-var 3-tuple sib, width stepped via var-name length.
#       C should read flat+2 (X flips at s=41), own budget 46 (flip 47, mod
#       the [45, b+1] relief).
#   PA: quote FIRST in 3-tuple  (X flip at 41-corr; own flip at 47+corr').
#   PB: quote MIDDLE in 3-tuple.
#   PC: quote LAST in 3-tuple.
#   PD: TWO quotes in 3-tuple.
#   PE: quote in a PAIR (dtop=0 control for the quote-in-tuple effect).
#   UEV: pure-var 5-elem union sib, stepped via var-name length — C(union)
#       hypothesis flat+2n-4 (n=5 => +6): X flips at s=37 if +6, 41 if +2,
#       43 if 0; own flip locates the union's self-budget bonus.
#   UG/DN: two more squeezed-fill points (union/chain vs argfact/atom sibs).
import sys

def pvar(i, ln=0):
    a = "abcdefghijklmnopqrstuvwxyz"
    base = a[i // 26] + a[i % 26]
    return "$" + base + "x" * max(0, ln - 3 - len(base) + 2)

def longvar(ln, tag):
    # display length ln incl '$': '$' + (ln-1) letters, tag keeps names unique
    body = ("q" + tag).ljust(ln - 1, "a")
    return "$" + body

def pvars(n, off=0):
    a = "abcdefghijklmnopqrstuvwxyz"
    return ["$" + a[(i + off) // 26] + a[(i + off) % 26] for i in range(n)]

def argfact(name, n, off=0):
    return f"{name}({', '.join(pvars(n, off))})"

class B:
    def __init__(s, thy, builtins=None):
        s.thy, s.builtins = thy, builtins
        s.rules, s.lemmas, s.names, s.n = [], [], [], 0
    def emit(s, name, prems, concls, comment):
        s.rules.append(f"// {comment}\nrule {name}:\n  [ {', '.join(prems)} ]\n"
                       f"  --[ F{s.n}() ]->\n  [ {', '.join(concls)} ]\n")
        s.lemmas.append(f"lemma l_{name}:\n  exists-trace \"Ex #i. F{s.n}() @ #i\"\n")
        s.names.append(name)
        s.n += 1
    def write(s, path):
        with open(path, "w") as f:
            f.write(f"theory {s.thy}\nbegin\n\n")
            if s.builtins:
                f.write(f"builtins: {s.builtins}\n\n")
            for r in s.rules:
                f.write(r + "\n")
            for l in s.lemmas:
                f.write(l + "\n")
            f.write("end\n")
        with open(path.replace(".spthy", ".names"), "w") as f:
            f.write("\n".join(s.names) + "\n")
        print(f"wrote {path}: {len(s.rules)} rules")

E = B("QuoteTuplePinProbe", builtins="multiset")
base = [argfact("Faa", 8)]                       # 45-flat argfact, off 0..7
prem = [f"In(<{', '.join(pvars(2, 100))}>)"]

# PF control: Qv(<$longvar, $ai, $aj>)  flat = 2+2+1+L+2+3+2+3+1+2 = L+16
for s in range(38, 48):
    L = s - 16
    E.emit(f"PF_{s:02d}", prem, base + [f"Qv(<{longvar(L, f'{E.n:02d}')}, {pvars(1,8)[0]}, {pvars(1,9)[0]}>)"],
           f"PF: [Faa 45, pure-var 3-tuple {s}] control")
# PA: quote FIRST: Qt(<'a..NN', $ai, $aj>)  flat = (content+2) + 18 = content + 20
for s in range(36, 48):
    E.emit(f"PA_{s:02d}", prem, base + [f"Qt(<'{'a' * (s - 22)}{E.n:02d}', {pvars(1,8)[0]}, {pvars(1,9)[0]}>)"],
           f"PA: [Faa 45, quote-first 3-tuple {s}]")
# PB: quote MIDDLE
for s in range(38, 45):
    E.emit(f"PB_{s:02d}", prem, base + [f"Qm(<{pvars(1,8)[0]}, '{'b' * (s - 22)}{E.n:02d}', {pvars(1,9)[0]}>)"],
           f"PB: [Faa 45, quote-middle 3-tuple {s}]")
# PC: quote LAST
for s in range(38, 45):
    E.emit(f"PC_{s:02d}", prem, base + [f"Qw(<{pvars(1,8)[0]}, {pvars(1,9)[0]}, '{'c' * (s - 22)}{E.n:02d}'>)"],
           f"PC: [Faa 45, quote-last 3-tuple {s}]")
# PD: TWO quotes: Q2(<'a..', 'd..4', $ai>)  flat = q1+2 + 8+2 + 3 + seps 4 + 2+2+2 = q1+...
#   inner = (q1)+(2)+ q2=8 +(2)+ 3 ; tuple = inner+2 ; fact = tuple+ 2+2+2? compute: 2+2+1+q1+2+8+2+3+1+2 = q1+23
for s in range(36, 44):
    E.emit(f"PD_{s:02d}", prem, base + [f"Q2(<'{'a' * (s - 27)}{E.n:02d}', 'dddd{E.n:02d}', {pvars(1,8)[0]}>)"],
           f"PD: [Faa 45, two-quote 3-tuple {s}]")
# PE: quote in PAIR (dtop 0): Qp(<'a..', $ai>) flat = 2+2+1+q+2+3+1+2 = q+13
for s in range(39, 46):
    E.emit(f"PE_{s:02d}", prem, base + [f"Qp(<'{'e' * (s - 17)}{E.n:02d}', {pvars(1,8)[0]}>)"],
           f"PE: [Faa 45, quote-in-pair {s}]")
# UEV: pure-var 5-elem union: Un(($lv++$ai++$aj++$ak++$al)) flat = 2+2+1+L+8+12+1+2 = L+28
for s in range(34, 50):
    L = s - 28
    E.emit(f"UEV_{s:02d}", prem, base + [f"Un(({longvar(L, f'{E.n:02d}')}++{'++'.join(pvars(4, 8))}))"],
           f"UEV: [Faa 45, pure-var 5-union {s}] — C and own-budget pin")
# UG/DN: extra squeezed-fill points
E.emit("UG_1", prem, [f"U({'++'.join(pvars(20))})", argfact("Fbb", 3, off=20)],
       "UG: [20-union 108, Fbb 20] fill point")
E.emit("UG_2", prem, [f"U({'++'.join(pvars(20))})", argfact("Fcc", 6, off=20)],
       "UG: [20-union 108, Fcc 35] fill point")
chain = pvars(1, 13)[0]
for i in range(12, -1, -1):
    chain = f"p({pvars(1, i)[0]}, {chain})"
E.emit("DN_2", prem, [f"Q({chain})", argfact("Fdd", 8, off=20)],
       "DN: [deep chain 112, Fdd 45] fill point")
E.rules.insert(0, "")  # spacing only
E.write("probeE.spthy")
# functions decl for the chain
txt = open("probeE.spthy").read().replace("builtins: multiset\n", "builtins: multiset\nfunctions: p/2\n")
open("probeE.spthy", "w").write(txt)
print("added functions: p/2")
