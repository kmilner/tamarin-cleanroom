#!/usr/bin/env python3
"""Round-12 batteries L/M/N/O.

L (probeL.spthy): pass-1 self-budget bonus for MULTI-ARG facts whose LAST arg
  is a tuple/union (family B).  Beside a floor-protected sib (flat 20, never
  wraps, C = 20; bonus-free budget 67), sweep the target flat across the
  boundary for tuple sizes n = 2,3,4,6, lead args 1..2, plus single-arg /
  mid-list / union controls.  Corpus hypothesis: multi-arg slack = fl(n/2)+1
  (one less than the single-arg fl(n/2)+2).
M (probeM.spthy): relief-charge law on PLAIN targets: [wrapping W, target T at
  the relief boundary].  Hybrid hypothesis: a wrapping sib charges its fill
  when deeply broken (fill < flat-4) else fill+1; W ~ 87+g puts the sib's
  break gap at g with T at the boundary.  Plus rec-sib cap rows (charge
  min(fill+1, C) vs C).
N (probeN.spthy): fill-numerator terms: nfunc exclusion (corpus: functions do
  NOT enter N), quoted-constant discount (corpus: nq-negative bias), rec cap
  at 7 for 7/8-elem receivers, and the fill-denominator charge for BIG-tuple
  sibs (10/16-elem: rec vs rec7).
O (probeO.spthy): trigger margins: all-plain rows with C-total 88/89 (which
  cells stay flat at budget+1), n=3 versions, corpus FW replicas (quote/pair
  rows), and the [41w, 51 deep-nested single-arg pair] FW witness replica.
"""
import math

def pvars(n, off=0):
    a = "abcdefghijklmnopqrstuvwxyz"
    return ["$" + a[(i + off) // 26] + a[(i + off) % 26] for i in range(n)]

def longvar(ln, tag):
    body = ("q" + tag).ljust(ln - 1, "a")
    assert len(body) + 1 == ln
    return "$" + body

def argfact(name, flat, tag, off):
    minL = 2 + len(tag)
    k = (flat - len(name) - 4 - minL) // 5
    if k < 0:
        k = 0
    L1 = flat - len(name) - 4 - 5 * k
    assert L1 >= minL, (name, flat, k, L1)
    elems = [longvar(L1, tag)] + pvars(k, off)
    s = f"{name}( {', '.join(elems)} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({', '.join(elems)})"

def pairfact(name, flat, tag, off, inner_n=2):
    fixed = len(name) + 2 + 1 + (inner_n - 1) * 5 + 1 + 2
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (name, flat, L1)
    elems = [longvar(L1, tag)] + pvars(inner_n - 1, off)
    s = f"{name}( <{', '.join(elems)}> )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(<{', '.join(elems)}>)"

def mtupfact(name, flat, tag, off, lead, inner_n):
    """`name( LONG, $aa[, ...], <$b1, ..., $bn> )` — lead plain args then a
    LAST tuple of inner_n 3-char vars."""
    tup = "<" + ", ".join(pvars(inner_n, off + 10)) + ">"
    fixed = len(name) + 4 + 5 * (lead - 1) + 2 + len(tup)
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (name, flat, L1)
    elems = [longvar(L1, tag)] + pvars(lead - 1, off) + [tup]
    s = f"{name}( {', '.join(elems)} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({', '.join(elems)})"

def midtupfact(name, flat, tag, off, inner_n):
    """`name( LONG, <...>, $aa )` — tuple mid-list (2nd of 3)."""
    tup = "<" + ", ".join(pvars(inner_n, off + 10)) + ">"
    fixed = len(name) + 4 + 2 + len(tup) + 5
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (name, flat, L1)
    s = f"{name}( {longvar(L1, tag)}, {tup}, {pvars(1, off)[0]} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({longvar(L1, tag)}, {tup}, {pvars(1, off)[0]})"



def munifact(name, flat, tag, off, inner_n):
    """`name( LONG, (a++b++c) )` — last arg an inner_n-union of 3-char vars."""
    uni = "(" + "++".join(pvars(inner_n, off + 10)) + ")"
    fixed = len(name) + 4 + 2 + len(uni)
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (name, flat, L1)
    s = f"{name}( {longvar(L1, tag)}, {uni} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({longvar(L1, tag)}, {uni})"

def funcfact(name, flat, tag, off, nfunc):
    """`name( LONG, w1($aa), ... )` — nfunc unary applications."""
    funcs = [f"w1({v})" for v in pvars(nfunc, off)]
    fixed = len(name) + 4 + sum(len(f) + 2 for f in funcs) - 2 + 2
    L1 = flat - len(name) - 4 - sum(len(f) + 2 for f in funcs)
    assert L1 >= 2 + len(tag), (name, flat, L1)
    elems = [longvar(L1, tag)] + funcs
    s = f"{name}( {', '.join(elems)} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({', '.join(elems)})"

def quotefact(name, flat, tag, off, nq):
    """`name( LONG, 'ab', 'cd', ... )` — nq quoted constants."""
    qs = ["'q%d%s'" % (i, "abcdefgh"[i]) for i in range(nq)]
    L1 = flat - len(name) - 4 - sum(len(q) + 2 for q in qs)
    assert L1 >= 2 + len(tag), (name, flat, L1)
    elems = [longvar(L1, tag)] + qs
    s = f"{name}( {', '.join(elems)} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({', '.join(elems)})"


def midpair(name, flat, tag):
    """`name( LONG, <$a, $b>, $c )` — compact mid-list pair (rec 3, 3 args)."""
    fixed = len(name) + 4 + 2 + 8 + 4
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (name, flat, L1)
    s = f"{name}( {longvar(L1, tag)}, <$a, $b>, $c )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({longvar(L1, tag)}, <$a, $b>, $c)"

def hd(x):
    fl = math.floor(x)
    return int(fl) if abs(x - fl - 0.5) < 1e-9 else int(math.floor(x + 0.5))

class B:
    def __init__(s, thy, extra=""):
        s.thy, s.extra = thy, extra
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
            if s.extra:
                f.write(s.extra + "\n")
            f.write("\n")
            for r in s.rules:
                f.write(r + "\n")
            for l in s.lemmas:
                f.write(l + "\n")
            f.write("end\n")
        with open(path.replace(".spthy", ".names"), "w") as f:
            f.write("\n".join(s.names) + "\n")
        print(f"wrote {path}: {len(s.rules)} rules")

prem = [f"In(<{', '.join(pvars(2, 100))}>)"]

# ---------------- L: multi-arg last-tuple bonus ----------------
L = B("BonusGateProbe", "builtins: multiset")
SIB = 20  # floor-protected sib, never wraps; bonus-free budget 67
for n, flats in ((2, (68, 69, 70, 71)), (3, (68, 69, 70, 71)),
                 (4, (69, 70, 71, 72, 73)), (6, (70, 71, 72, 73))):
    for fl in flats:
        L.emit(f"LA{n}_{fl}", prem,
               [argfact("Fzz", SIB, f"{L.n:02d}", 0),
                mtupfact("Mtt", fl, f"x{L.n:02d}", 30, 2, n)],
               f"LA: multi-arg(2 lead) last-{n}-tuple flat {fl} beside flat-20 (budget0 67)")
for n, flats in ((2, (69, 70, 71)), (4, (70, 71, 72))):
    for fl in flats:
        L.emit(f"LB{n}_{fl}", prem,
               [argfact("Fzz", SIB, f"{L.n:02d}", 0),
                mtupfact("Mtt", fl, f"x{L.n:02d}", 30, 1, n)],
               f"LB: multi-arg(1 lead) last-{n}-tuple flat {fl}")
for fl in (69, 70, 71):
    L.emit(f"LC3_{fl}", prem,
           [argfact("Fzz", SIB, f"{L.n:02d}", 0),
            pairfact("Stt", fl, f"x{L.n:02d}", 30, 3)],
           f"LC: single-arg 3-tuple control flat {fl} (expect fits <= 70)")
for fl in (67, 68, 69):
    L.emit(f"LD4_{fl}", prem,
           [argfact("Fzz", SIB, f"{L.n:02d}", 0),
            midtupfact("Dtt", fl, f"x{L.n:02d}", 30, 4)],
           f"LD: mid-list 4-tuple control flat {fl} (expect fits <= 67)")
for fl in (68, 69, 70):
    L.emit(f"LE3_{fl}", prem,
           [argfact("Fzz", SIB, f"{L.n:02d}", 0),
            munifact("Utt", fl, f"x{L.n:02d}", 30, 3)],
           f"LE: multi-arg last-3-union flat {fl}")
L.write("probeL.spthy")

# ---------------- M: relief-charge law ----------------
# Plain-target rows are DEGENERATE (the proportional fill couples the relief
# boundary onto the pass-1 boundary: f(W + 87 - f) = 87W forces f in {W, 87});
# a REC-carrying target (mid-list pair, rec 3, bonus 0 by WIT) breaks the
# coupling.  Target T (midpair) beside plain wrapping sib W with fill f and
# gap g = W - f: hybrid predicts T saved iff T <= 87 - (f if g >= 5 else f+1).
M = B("ReliefChargeProbe")
def clampf(b, w):
    return max(20, min(b, max(w - 1, 20)))
seen_g = set()
for W in range(46, 110):
    # boundary T for charge=f: largest T with T <= 87 - f(W,T)
    best = None
    for T in range(21, 70):
        f = clampf(hd(87.0 * W / (W + T + 3)), W)
        if not (T > 87 - W):        # target must FAIL pass 1 (bonus 0)
            continue
        if not (W > 87 - (T + 3)):  # sib must wrap
            continue
        if T <= 87 - f:
            best = (T, f)
    if best is None:
        continue
    T, f = best
    g = W - f
    if g in seen_g or not 1 <= g <= 9:
        continue
    seen_g.add(g)
    for dt in (-1, 0, 1):
        if T + dt <= 87 - W or T + dt < 28:
            continue
        M.emit(f"MA{g}_{W}_{T + dt}", prem,
               [argfact("Wbb", W, f"{M.n:02d}", 0),
                midpair("Tc", T + dt, f"z{M.n}")],
               f"MA: gap-{g} sib [W {W} fill~{f}], midpair T {T + dt} (charge-f boundary T {T})")
# MC: pair-SIB (C = N = W+3) beside midpair target: at shallow gap the hybrid
# charges f+1 but t2 charges C = W+3 (differ by g+2).
for W in (52, 56, 60, 66):
    best = None
    for T in range(21, 70):
        f = clampf(hd(87.0 * (W + 3) / (W + 3 + T + 3)), W)
        if not (T > 87 - (W + 3)) or not (W > 87 - (T + 3)):
            continue
        if T <= 87 - (f + 1):
            best = (T, f)
    if best is None:
        continue
    T, f = best
    for dt in (0, 1):
        if T + dt < 28:
            continue
        M.emit(f"MC_{W}_{T + dt}", prem,
               [pairfact("Pbb", W, f"{M.n:02d}", 0, 2),
                midpair("Tc", T + dt, f"z{M.n}")],
               f"MC: pair-sib [P {W} rec3 fill~{f} gap {W - f}], midpair T {T + dt} (f+1 boundary T {T})")
M.write("probeM.spthy")

# ---------------- N: fill numerator / denominator ----------------
N = B("FillTermsProbe", "functions: w1/1")
def pick_rows(mk_target, nlab, nvals, svals, numf, numf_alt, crange=3):
    """Emit rows where hd of the two numerator laws differ."""
    out = []
    for t in nvals:
        for s in svals:
            n1 = numf(t)
            n2 = numf_alt(t)
            b1 = max(20, min(hd(87.0 * n1 / (n1 + s)), t - 1))
            b2 = max(20, min(hd(87.0 * n2 / (n2 + s)), t - 1))
            if b1 != b2 and t > 87 - s and s > 87 - t:  # both wrap (plain sib)
                out.append((t, s, b1, b2))
    return out[:crange]
# NA: nfunc 2 and 4 — N=flat (f7) vs N=flat+nfunc (f7n)
for nf in (2, 4):
    cnt = 0
    for t in range(58, 78):
        for s in range(40, 72, 1):
            b_f7 = max(20, min(hd(87.0 * t / (t + s)), t - 1))
            b_f7n = max(20, min(hd(87.0 * (t + nf) / (t + nf + s)), t - 1))
            if b_f7 != b_f7n and t > 87 - s and s > 87 - t and cnt < 4:
                N.emit(f"NA{nf}_{t}_{s}", prem,
                       [funcfact("Ftt", t, f"{N.n:02d}", 0, nf), argfact("Sbb", s, f"x{N.n:02d}", 30)],
                       f"NA: {nf}-func target {t} beside {s}: f7 fill {b_f7} vs f7n {b_f7n}")
                cnt += 1
# NB: nq 2 and 4 — N=flat vs flat-1q vs flat-2q
for nq in (2, 4):
    cnt = 0
    for t in range(58, 78):
        for s in range(40, 72):
            b0 = max(20, min(hd(87.0 * t / (t + s)), t - 1))
            b1 = max(20, min(hd(87.0 * (t - nq) / (t - nq + s)), t - 1))
            b2 = max(20, min(hd(87.0 * (t - 2 * nq) / (t - 2 * nq + s)), t - 1))
            if len({b0, b1, b2}) == 3 and t > 87 - s and s > 87 - t and cnt < 4:
                N.emit(f"NB{nq}_{t}_{s}", prem,
                       [quotefact("Qtt", t, f"{N.n:02d}", 0, nq), argfact("Sbb", s, f"x{N.n:02d}", 30)],
                       f"NB: {nq}-quote target {t} beside {s}: fills q0 {b0} / q-1 {b1} / q-2 {b2}")
                cnt += 1
# NC: 7- and 8-elem tuple receivers — rec cap 7 vs elems+1
for ne in (7, 8):
    cnt = 0
    sur_cap, sur_full = 7, ne + 1
    for t in range(62, 82):
        for s in range(40, 72):
            bc = max(20, min(hd(87.0 * (t + sur_cap) / (t + sur_cap + s)), t - 1))
            bf = max(20, min(hd(87.0 * (t + sur_full) / (t + sur_full + s)), t - 1))
            if bc != bf and t > 87 - s + 4 and s > 87 - (t + sur_full) and cnt < 4:
                N.emit(f"NC{ne}_{t}_{s}", prem,
                       [pairfact("Rtt", t, f"{N.n:02d}", 0, ne), argfact("Sbb", s, f"x{N.n:02d}", 30)],
                       f"NC: {ne}-tuple receiver {t} beside {s}: cap7 fill {bc} vs full {bf}")
                cnt += 1
# ND: plain target beside BIG-tuple sib (10/16-elem): denominator C vs C7
for ne in (10, 16):
    cnt = 0
    rec, rec7 = ne + 1, 7
    for t in range(50, 80):
        for s in range(60 if ne == 10 else 90, 115):
            bC = max(20, min(hd(87.0 * t / (t + s + rec)), t - 1))
            bC7 = max(20, min(hd(87.0 * t / (t + s + rec7)), t - 1))
            if bC != bC7 and t > 87 - (s + rec) and s > 87 - (t + 0) and cnt < 4:
                N.emit(f"ND{ne}_{t}_{s}", prem,
                       [argfact("Ttt", t, f"{N.n:02d}", 0), pairfact("Gbb", s, f"x{N.n:02d}", 30, ne)],
                       f"ND: plain {t} beside {ne}-tuple sib {s}: D=C fill {bC} vs C7 {bC7}")
                cnt += 1
N.write("probeN.spthy")

# ---------------- O: trigger margins ----------------
O = B("TriggerMarginProbe", "functions: w1/1")
for a, b in ((45, 43), (46, 42), (44, 44), (47, 41), (50, 38), (55, 33),
             (60, 28), (65, 23), (45, 44), (46, 43), (50, 39)):
    O.emit(f"OA_{a}_{b}", prem,
           [argfact("Aaa", a, f"{O.n:02d}", 0), argfact("Bbb", b, f"x{O.n:02d}", 30)],
           f"OA: plain pair [{a}, {b}] total {a + b}")
for tri in ((20, 39, 29), (30, 30, 28), (40, 24, 24), (29, 30, 29)):
    a, b, c = tri
    O.emit(f"OB_{a}_{b}_{c}", prem,
           [argfact("Aaa", a, f"{O.n:02d}", 0), argfact("Bbb", b, f"x{O.n:02d}", 30),
            argfact("Ccc", c, f"y{O.n:02d}", 60)],
           f"OB: plain triple {tri} total {sum(tri)}")
# OC: corpus FW replica [pair34(rec4? use 3-tuple: rec 4), 22, pair25]: sweep mid
for mid in (21, 22, 23, 24):
    O.emit(f"OC_{mid}", prem,
           [pairfact("Paa", 34, f"{O.n:02d}", 0, 3), argfact("Tbb", mid, f"x{O.n:02d}", 30),
            pairfact("Qcc", 25, f"y{O.n:02d}", 60, 2)],
           f"OC: FW replica [34(3-tup rec4), {mid}, 25(pair rec3)] — ref keeps {mid} flat?")
# OD: [41w mid-pair+func, 51 deep-nested single-arg pair] replica; sweep target
def deeppair(name, flat, tag, off):
    """`name( <w1(<LONG, $b, $c>), <$d, $e>> )` — rec 3+4+1 = 8, nfunc 1."""
    core = "<w1(<LNG, $f, $g>), <$h, $i>>"
    L1 = flat - (len(name) + 4 + len(core) - 3)
    assert L1 >= 2 + len(tag), (name, flat, L1)
    core = core.replace("LNG", longvar(L1, tag))
    s = f"{name}( {core} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({core})"
def midpairfunc(name, flat, tag, off):
    """`name( LONG, $a, <$b, $c>, w1($d), $e )` — rec 3, nfunc 1, nargs 5."""
    tail = "$a, <$b, $c>, w1($d), $e"
    L1 = flat - (len(name) + 4 + len(tail) + 2)
    assert L1 >= 2 + len(tag), (name, flat, L1)
    s = f"{name}( {longvar(L1, tag)}, {tail} )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({longvar(L1, tag)}, {tail})"
for t in (49, 50, 51, 52, 53):
    O.emit(f"OD_{t}", prem,
           [midpairfunc("Ann", 41, f"{O.n:02d}", 0), deeppair("Bnn", t, f"x{O.n:02d}", 5)],
           f"OD: [41(mid-pair+func rec3), deep-pair {t} (rec8, nfunc1)] — FW witness replica")
O.write("probeO.spthy")
