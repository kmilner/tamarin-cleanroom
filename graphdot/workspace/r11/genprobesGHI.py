#!/usr/bin/env python3
"""Round-11 batteries G/H/I: multi-cell fill allocation + wrap relief.

G (probeG.spthy): 2-cell conclusion rows where BOTH cells wrap — coarse grid of
  (narrow, wide) flats + near-equal pairs + corpus witness controls.  Reads the
  per-cell allocated ribbons via band extraction (the round-10 under-sampled
  interaction: every prior fill battery had fitting/unbreakable siblings).
H (probeH.spthy): 3- and 4-cell rows mixing wrapping and flat cells — pins how
  a wrapping sibling is charged in the others' fills (post-wrap alloc vs C).
I (probeI.spthy): family-3 trigger relief — a target cell T beside a WRAPPING
  wide sibling W: sweep T's flat across the boundary to find where T stops
  fitting, as a function of W's flat; with tuple-carrying T variants and a
  3-cell variant.

All cells are plain argfacts of distinct $-vars (one longvar to hit an exact
flat + 3-char pvars for tight bands); no term repeats within a rule, so no
abbreviation fires (len>=10 needs occ>=2)."""

def pvars(n, off=0):
    a = "abcdefghijklmnopqrstuvwxyz"
    return ["$" + a[(i + off) // 26] + a[(i + off) % 26] for i in range(n)]

def longvar(ln, tag):
    body = ("q" + tag).ljust(ln - 1, "a")
    return "$" + body

def argfact(name, flat, tag, off):
    """`name( LONG, $aa, $ab, ... )` of exact display width `flat`."""
    minL = 2 + len(tag)  # "$q<tag>" cannot shrink below this
    k = (flat - len(name) - 4 - minL) // 5
    if k < 0:
        k = 0
    L1 = flat - len(name) - 4 - 5 * k
    assert L1 >= minL, (name, flat, k, L1)
    elems = [longvar(L1, tag)] + pvars(k, off)
    s = f"{name}( {', '.join(elems)} )"
    # rendered form pads exactly like this; sanity-check width
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}({', '.join(elems)})"

def pairfact(name, flat, tag, off, inner_n=2):
    """`name( <LONG, $aa[, ...]> )` — one top-level tuple arg (tup_sur =
    inner_n+1)."""
    fixed = len(name) + 2 + 1 + (inner_n - 1) * 5 + 1 + 2  # name( <...> )
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (name, flat, L1)
    elems = [longvar(L1, tag)] + pvars(inner_n - 1, off)
    s = f"{name}( <{', '.join(elems)}> )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(<{', '.join(elems)}>)"

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

prem = [f"In(<{', '.join(pvars(2, 100))}>)"]

# ---------------- G ----------------
G = B("FillPairProbe")
for nn in (30, 40, 45, 50, 55, 61, 66, 71, 76, 81):
    for ww in (90, 105, 120):
        G.emit(f"GA_{nn}_{ww}", prem,
               [argfact("Naa", nn, f"{G.n:02d}", 0), argfact("Wbb", ww, f"x{G.n:02d}", 30)],
               f"GA: both-wrap pair [{nn}, {ww}]")
for (a, b) in ((45, 45), (50, 50), (55, 55), (60, 60), (70, 70), (80, 80),
               (60, 70), (50, 70), (45, 60), (45, 90)):
    G.emit(f"GB_{a}_{b}", prem,
           [argfact("Naa", a, f"{G.n:02d}", 0), argfact("Wbb", b, f"x{G.n:02d}", 30)],
           f"GB: near-equal both-wrap pair [{a}, {b}]")
# corpus witness controls: [42 plain, 116 pair-tuple], [61 plain, 114 pair-tuple],
# [75 plain, 27 pair-tuple]
G.emit("GC_42_116", prem,
       [argfact("Rem", 42, f"{G.n:02d}", 0), pairfact("Ott", 116, f"x{G.n:02d}", 30, 6)],
       "GC: witness [42, 116(tuple)]")
G.emit("GC_61_114", prem,
       [argfact("Rem", 61, f"{G.n:02d}", 0), pairfact("Ott", 114, f"x{G.n:02d}", 30, 6)],
       "GC: witness [61, 114(tuple)]")
G.emit("GC_75_27", prem,
       [argfact("Rem", 75, f"{G.n:02d}", 0), pairfact("Pai", 27, f"x{G.n:02d}", 30, 2)],
       "GC: witness [75, 27(pair)]")
G.write("probeG.spthy")

# ---------------- H ----------------
H = B("FillTripleProbe")
for ww in (70, 90):
    for mm in (26, 35, 45):
        for ss in (16, 19):
            H.emit(f"HA_{ww}_{mm}_{ss}", prem,
                   [argfact("Waa", ww, f"{H.n:02d}", 0), argfact("Mbb", mm, f"x{H.n:02d}", 30),
                    argfact("Scc", ss, f"y{H.n:02d}", 60)],
                   f"HA: [W {ww}, M {mm}, S {ss}] wrap/wrap(?)/flat")
for (a, b, c) in ((45, 32, 27), (45, 33, 26), (41, 30, 23), (35, 29, 30),
                  (50, 40, 30), (60, 45, 35)):
    H.emit(f"HC_{a}_{b}_{c}", prem,
           [argfact("Aaa", a, f"{H.n:02d}", 0), argfact("Bbb", b, f"x{H.n:02d}", 30),
            argfact("Ccc", c, f"y{H.n:02d}", 60)],
           f"HC: all-wrap triple [{a}, {b}, {c}]")
for (a, b, c) in ((54, 61, 19), (52, 59, 17), (70, 26, 16), (72, 26, 19), (51, 26, 15)):
    H.emit(f"HD_{a}_{b}_{c}", prem,
           [argfact("Aaa", a, f"{H.n:02d}", 0), argfact("Bbb", b, f"x{H.n:02d}", 30),
            argfact("Ccc", c, f"y{H.n:02d}", 60)],
           f"HD: two-wide + small [{a}, {b}, {c}]")
for (a, b, c, d) in ((45, 30, 25, 20), (60, 40, 30, 16)):
    H.emit(f"HE_{a}_{b}_{c}_{d}", prem,
           [argfact("Aaa", a, f"{H.n:02d}", 0), argfact("Bbb", b, f"x{H.n:02d}", 30),
            argfact("Ccc", c, f"y{H.n:02d}", 60), argfact("Ddd", d, f"z{H.n:02d}", 90)],
           f"HE: 4-cell [{a}, {b}, {c}, {d}]")
H.write("probeH.spthy")

# ---------------- I ----------------
I = B("ReliefProbe")
sweeps = {65: range(21, 31), 75: range(22, 28), 90: range(19, 29),
          100: range(22, 28), 116: range(19, 29)}
for ww, ts in sweeps.items():
    for tt in ts:
        I.emit(f"IA_{ww}_{tt}", prem,
               [argfact("Waa", ww, f"{I.n:02d}", 0), argfact("Tbb", tt, f"x{I.n:02d}", 30)],
               f"IA: relief boundary [W {ww} wraps, T {tt}]")
for tt in range(20, 28):
    I.emit(f"IB_90_{tt}", prem,
           [argfact("Waa", 90, f"{I.n:02d}", 0), pairfact("Tpp", tt, f"x{I.n:02d}", 30, 2)],
           f"IB: relief with pair-tuple target [W 90, Tpair {tt}]")
for tt in range(20, 26):
    I.emit(f"IC_90_{tt}_16", prem,
           [argfact("Waa", 90, f"{I.n:02d}", 0), argfact("Tbb", tt, f"x{I.n:02d}", 30),
            argfact("Scc", 16, f"y{I.n:02d}", 60)],
           f"IC: 3-cell relief [W 90, T {tt}, S 16]")
I.write("probeI.spthy")
