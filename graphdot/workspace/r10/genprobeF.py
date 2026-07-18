#!/usr/bin/env python3
# Round-10 battery F: size laws for tuple/union corrections.
#   TN: [Faa 45, pure-var n-tuple stepped] X-flip pins corr(n); n = 2, 3, 4, 6.
#       corr = n+1 predicts X first wraps at sib flat 43-corr+1 = 40/39/38/36;
#       corr = 2n-4 predicts 43/41/39/35.
#   TB: [n-tuple stepped, Fbb 45] own-flip pins bonus(n); n = 2, 6
#       (battery E gave bonus(3-tuple) = 3, bonus(5-union) = 5; bonus = n
#       predicts first own-wrap at flat 45/49).
#   UN: [Faa 45, n-union stepped] X-flip pins union corr(n); n = 3, 8
#       (E: corr(5-union) = 6; n+1 vs 2n-4 split at n=3: 4 vs 2, n=8: 9 vs 12).
#   UB2: [8-union stepped, Fbb 45] own-flip pins union bonus(8).
# Narrow shapes use bare 2-/1-char message vars bound by an extra In premise
# (the r8 pattern); premise-row width does not couple into the conclusion row.
def pvars(n, off=0):
    a = "abcdefghijklmnopqrstuvwxyz"
    return ["$" + a[(i + off) // 26] + a[(i + off) % 26] for i in range(n)]

def longvar(ln, tag):
    body = ("q" + tag).ljust(ln - 1, "a")
    return "$" + body

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
                f.write(f"builtins: {s.builtins}\n")
            f.write("\n")
            for r in s.rules:
                f.write(r + "\n")
            for l in s.lemmas:
                f.write(l + "\n")
            f.write("end\n")
        with open(path.replace(".spthy", ".names"), "w") as f:
            f.write("\n".join(s.names) + "\n")
        print(f"wrote {path}: {len(s.rules)} rules")

F = B("SizeLawProbe", builtins="multiset")
base = [argfact("Faa", 8)]                        # 45-flat argfact, off 0..7
prem = [f"In(<{', '.join(pvars(2, 100))}>)"]

def tuple_fact(n, s, tag, nbare=0):
    """Pure-var n-tuple fact T<n>(<...>) of display width s: one longvar,
    (n-1-nbare) $-vars, nbare bare 2-char vars (zz, zy, ...)."""
    bares = ["zz", "zy", "zx", "zw"][:nbare]
    fixed = (n - 1 - nbare) * 3 + nbare * 2
    L = s - (2 + 2 + 1 + fixed + 2 * (n - 1) + 1 + 2)
    elems = [longvar(L, tag)] + pvars(n - 1 - nbare, 8) + bares
    return f"T{n}(<{', '.join(elems)}>)", bares

def union_fact(n, s, tag, nbare=0):
    bares = ["z", "y", "x", "w", "v", "u", "t"][:nbare]
    fixed = (n - 1 - nbare) * 3 + nbare * 1
    L = s - (2 + 2 + 1 + fixed + 2 * (n - 1) + 1 + 2)
    elems = [longvar(L, tag)] + pvars(n - 1 - nbare, 8) + bares
    return f"U{n}(({'++'.join(elems)}))", bares

def with_bares(bares):
    if not bares:
        return prem
    return prem + [f"In(<{', '.join(bares)}>)"]

# TN: X-flip sweeps (sib = the shaped cell, X = Faa 45)
for (n, lo, hi, nb) in [(2, 37, 43, 0), (3, 37, 40, 0), (4, 35, 41, 0), (6, 33, 39, 4)]:
    for s in range(lo, hi + 1):
        t, bares = tuple_fact(n, s, f"{F.n:02d}", nb)
        F.emit(f"TN{n}_{s:02d}", with_bares(bares), base + [t],
               f"TN: [Faa 45, {n}-tuple {s}] X-flip pins corr({n})")
# TB: own-flip sweeps (X = the tuple, partner Fbb 45)
for (n, lo, hi) in [(2, 42, 47), (6, 46, 51)]:
    for s in range(lo, hi + 1):
        t, bares = tuple_fact(n, s, f"{F.n:02d}", 0)
        F.emit(f"TB{n}_{s:02d}", prem, [t, argfact("Fbb", 8, off=20)],
               f"TB: [{n}-tuple {s}, Fbb 45] own-flip pins bonus({n})")
# UN: union X-flips
for (n, lo, hi, nb) in [(3, 38, 43, 0), (8, 33, 38, 7)]:
    for s in range(lo, hi + 1):
        u, bares = union_fact(n, s, f"{F.n:02d}", nb)
        F.emit(f"UN{n}_{s:02d}", with_bares(bares), base + [u],
               f"UN: [Faa 45, {n}-union {s}] X-flip pins union corr({n})")
# UB2: union own-flip, n=8 pure $-vars
for s in range(48, 54):
    u, bares = union_fact(8, s, f"{F.n:02d}", 0)
    F.emit(f"UB8_{s:02d}", prem, [u, argfact("Fbb", 8, off=20)],
           f"UB2: [8-union {s}, Fbb 45] own-flip pins union bonus(8)")
F.write("probeF.spthy")
