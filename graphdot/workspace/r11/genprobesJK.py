#!/usr/bin/env python3
"""Round-11 batteries J/K.

J (probeJ.spthy): abbreviation internal-layout (families 2+4).  Each rule
  builds a state fact carrying an abbreviable term T = h('a...a') (len >= 10,
  occurs >= 2 in the system via producer conclusion + consumer premise), so the
  graph abbreviates T to H1 and the record cells show the abbreviated display
  text.  Captured TWICE: default and ?unabbreviate= (internal ground truth).
K (probeK.spthy): shape pins -- nested-tuple occupancy (trigger), pair/6-tuple
  receiver fill numerators vs sibling sweep, opener hangs for tuple-in-func /
  union / wide-first-element tuples, and exact pair-of-tuples witness replicas."""

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

class B:
    def __init__(s, thy, builtins=None):
        s.thy, s.builtins = thy, builtins
        s.rules, s.lemmas, s.names, s.n = [], [], [], 0
    def emit_raw(s, name, body, lemma):
        s.rules.append(body)
        s.lemmas.append(lemma)
        s.names.append(name)
        s.n += 1
    def emit(s, name, prems, concls, comment):
        body = (f"// {comment}\nrule {name}:\n  [ {', '.join(prems)} ]\n"
                f"  --[ F{s.n}() ]->\n  [ {', '.join(concls)} ]\n")
        lemma = f"lemma l_{name}:\n  exists-trace \"Ex #i. F{s.n}() @ #i\"\n"
        s.emit_raw(name, body, lemma)
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

# ---------------- J ----------------
# T(clen) = h('a…a') with a quoted constant of length clen; internal rendered
# width of h('c') = 4 + clen.  A producer rule makes St(~id, T[, extras]) and
# the consumer J rule consumes it; T occurs twice (producer concl + consumer
# prem, plus Out), so it abbreviates (len >= 10, occ >= 2, not a tuple).
J = B("AbbrLayoutProbe", builtins="hashing")

def qconst(clen, tag):
    return "'" + ("j" + tag).ljust(clen, "a") + "'"

def jrule(name, mkterm, extras_prod, comment):
    """Producer Init_<name> makes St_<name>(~id, T, extras); consumer <name>
    consumes it and re-emits Out(T)."""
    T = mkterm
    prod = (f"// {comment} (producer)\nrule Init_{name}:\n  [ Fr(~id) ]\n  -->\n"
            f"  [ St_{name}(~id, {T}{extras_prod}) ]\n")
    cons = (f"// {comment} (consumer)\nrule Go_{name}:\n"
            f"  [ St_{name}(~id, {T}{extras_prod}) ]\n  --[ F{J.n}() ]->\n"
            f"  [ Out({T}) ]\n")
    lemma = f"lemma l_Go_{name}:\n  exists-trace \"Ex #i. F{J.n}() @ #i\"\n"
    J.emit_raw(f"Go_{name}", prod + "\n" + cons, lemma)

# J1: internal ~96 (one break), abbr at END; display ~ St(~id, H1) ~ 17
jrule("J1", f"h({qconst(85, '01')})", "", "J1: lone cell internal 96+, display small")
# J2: internal ~150 (multiple breaks)
jrule("J2", f"h({qconst(140, '02')})", "", "J2: internal ~150")
# J3: abbr at START of the arg list (extras after)
J.rules.append("")  # spacing no-op
prodT = f"h({qconst(80, '03')})"
J.emit_raw("Go_J3",
    f"// J3: abbr first, vars after\nrule Init_J3:\n  [ Fr(~id) ]\n  -->\n"
    f"  [ St_J3({prodT}, ~id, $ea, $eb) ]\n\n"
    f"rule Go_J3:\n  [ St_J3({prodT}, ~id, $ea, $eb) ]\n  --[ F{J.n}() ]->\n"
    f"  [ Out({prodT}) ]\n",
    f"lemma l_Go_J3:\n  exists-trace \"Ex #i. F{J.n}() @ #i\"\n")
# J4: two abbreviated terms in one cell
Ta = f"h({qconst(45, '04')})"
Tb = f"h({qconst(45, '05')})"
J.emit_raw("Go_J4",
    f"// J4: two abbr terms\nrule Init_J4:\n  [ Fr(~id) ]\n  -->\n"
    f"  [ St_J4(~id, {Ta}, {Tb}) ]\n\n"
    f"rule Go_J4:\n  [ St_J4(~id, {Ta}, {Tb}) ]\n  --[ F{J.n}() ]->\n"
    f"  [ Out(<{Ta}, {Tb}>) ]\n",
    f"lemma l_Go_J4:\n  exists-trace \"Ex #i. F{J.n}() @ #i\"\n")
# J5: nested abbreviation: outer h(<T1, 'bb'>) with inner T1 abbreviated too
T1 = f"h({qconst(40, '06')})"
T2 = f"h(<{T1}, 'bbccddee'>)"
J.emit_raw("Go_J5",
    f"// J5: nested abbr\nrule Init_J5:\n  [ Fr(~id) ]\n  -->\n"
    f"  [ St_J5(~id, {T2}) ]\n\n"
    f"rule Go_J5:\n  [ St_J5(~id, {T2}) ]\n  --[ F{J.n}() ]->\n"
    f"  [ Out(<{T2}, {T1}>) ]\n",
    f"lemma l_Go_J5:\n  exists-trace \"Ex #i. F{J.n}() @ #i\"\n")
# J6: multi-cell row: abbr cell (display small, internal big) + Faa 45 sibling:
# does the sibling's budget see display-C or internal-C?
T6 = f"h({qconst(80, '07')})"
faa = argfact("Faa", 45, "08", 0)
J.emit_raw("Go_J6",
    f"// J6: [abbr cell, Faa 45] sibling budget probe\nrule Init_J6:\n  [ Fr(~id) ]\n  -->\n"
    f"  [ St_J6(~id, {T6}), Sd_J6(~id) ]\n\n"
    f"rule Go_J6:\n  [ St_J6(~id, {T6}), Sd_J6(~id), In(<$dw, $dx>) ]\n  --[ F{J.n}() ]->\n"
    f"  [ Out(<{T6}, $dw>), {faa} ]\n",
    f"lemma l_Go_J6:\n  exists-trace \"Ex #i. F{J.n}() @ #i\"\n")
# J7: family-4 witness replica: display ~78 (< 87), internal ~120, abbr at END
T7 = f"h({qconst(50, '09')})"
J.emit_raw("Go_J7",
    f"// J7: display ~78 internal ~120, abbr last\nrule Init_J7:\n  [ Fr(~id) ]\n  -->\n"
    f"  [ St_J7(~id, $fa, $fb, <'commit', $fc, $fd, $fe, $ff, $fg, $fh>, {T7}) ]\n\n"
    f"rule Go_J7:\n  [ St_J7(~id, $fa, $fb, <'commit', $fc, $fd, $fe, $ff, $fg, $fh>, {T7}) ]\n"
    f"  --[ F{J.n}() ]->\n  [ Out({T7}) ]\n",
    f"lemma l_Go_J7:\n  exists-trace \"Ex #i. F{J.n}() @ #i\"\n")
# J8: expansion spanning an internal break: T8 internal 60 placed after 50
# columns of vars, so the internal layout breaks inside T8's expansion
T8 = f"h({qconst(55, '10')})"
J.emit_raw("Go_J8",
    f"// J8: internal break lands inside the expansion\nrule Init_J8:\n  [ Fr(~id) ]\n  -->\n"
    f"  [ St_J8(~id, $ga, $gb, $gc, $gd, $ge, $gf, $gg, $gh, {T8}, $gi) ]\n\n"
    f"rule Go_J8:\n  [ St_J8(~id, $ga, $gb, $gc, $gd, $ge, $gf, $gg, $gh, {T8}, $gi) ]\n"
    f"  --[ F{J.n}() ]->\n  [ Out({T8}) ]\n",
    f"lemma l_Go_J8:\n  exists-trace \"Ex #i. F{J.n}() @ #i\"\n")
J.write("probeJ.spthy")

# ---------------- K ----------------
K = B("ShapePinProbe", builtins="multiset")
K.rules.append("functions: ff/1, w3/3, w1/1\n")
K.names_noop = None
# K1: nested-tuple occupancy: [Faa 45, N( <<LONG, $aa>, <$ab, $ac>> ) stepped]
# outer pair -> top-level +3; nested pairs would add +3 each if recursive
# (flip at s=40 top-only vs s=34 recursive).
def pairpair(name, flat, tag, off):
    fixed = len(name) + 2 + 2 + 5 + 1 + 2 + 10 + 1 + 2
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (flat, L1)
    a = pvars(3, off)
    s = f"{name}( <<{longvar(L1, tag)}, {a[0]}>, <{a[1]}, {a[2]}>> )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(<<{longvar(L1, tag)}, {a[0]}>, <{a[1]}, {a[2]}>>)"

for s in range(33, 42):
    K.emit(f"K1_{s}", prem, [argfact("Faa", 45, f"{K.n:02d}", 0),
                             pairpair("N", s, f"x{K.n:02d}", 30)],
           f"K1: [Faa 45, pair-of-pairs {s}] X-flip pins nested occupancy")
# K2: tuple nested in a FUNC arg: [Faa 45, Q( ff(<LONG, $aa, $ab>) ) stepped]
# top-level: no tuple arg -> C = s (flip 43); recursive tuple -> +4 (flip 39).
def functup(name, flat, tag, off):
    fixed = len(name) + 2 + 3 + 1 + 10 + 1 + 1 + 2
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (flat, L1)
    a = pvars(2, off)
    s = f"{name}( ff(<{longvar(L1, tag)}, {a[0]}, {a[1]}>) )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(ff(<{longvar(L1, tag)}, {a[0]}, {a[1]}>))"
for s in range(36, 45):
    K.emit(f"K2_{s}", prem, [argfact("Faa", 45, f"{K.n:02d}", 0),
                             functup("Q", s, f"x{K.n:02d}", 30)],
           f"K2: [Faa 45, func(3-tuple) {s}] X-flip pins func-nested occupancy")
# K3: tuple-receiver FILL numerator: pair + 6-tuple receivers vs sibling sweep
def ntup(name, flat, tag, off, n):
    fixed = len(name) + 2 + 1 + (n - 1) * 5 + 1 + 2
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (flat, L1)
    elems = [longvar(L1, tag)] + pvars(n - 1, off)
    s = f"{name}( <{', '.join(elems)}> )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(<{', '.join(elems)}>)"
for (rf, n) in ((27, 2), (36, 2), (40, 6)):
    for s in (50, 60, 70, 75, 80, 90):
        K.emit(f"K3_{rf}_{n}_{s}", prem,
               [ntup("Trr", rf, f"{K.n:02d}", 0, n), argfact("Sbb", s, f"x{K.n:02d}", 30)],
               f"K3: [{n}-tuple {rf}, Sib {s}] receiver fill numerator")
# K4: opener hangs: lone cells
K.emit("K4_tupfunc", prem, [f"Qzz(w1(<{longvar(84, '90')}, $ha, $hb, $hc>))"],
       "K4: lone func(tuple with 84-wide first elem) - hang shape?")
K.emit("K4_union", prem, [f"Uzz(({longvar(84, '91')}++$hd++$he))"],
       "K4: lone union with 84-wide first elem - hang after ( ?")
K.emit("K4_funcatom", prem, [f"Qzz(w3({longvar(84, '92')}, $hf, $hg))"],
       "K4: lone func with 84-wide first ARG - hang after name( ?")
K.emit("K4_tuple2", prem, [f"Tzz(<{longvar(95, '93')}, $hh>)"],
       "K4: lone tuple 95-wide first elem (control, GC shape)")
# K6: exact witness replicas: pair-of-6-tuples wide cell
def pair6(name, flat, tag, off):
    a = pvars(10, off)
    inner2 = ", ".join(a[5:10])
    fixed = len(name) + 2 + 1 + 1 + 5*5 + 1 + 2 + 1 + len(inner2) + 1 + 1 + 2
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (flat, L1)
    inner1 = ", ".join([longvar(L1, tag)] + a[0:5])
    s = f"{name}( <<{inner1}>, <{inner2}>> )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(<<{inner1}>, <{inner2}>>)"
K.emit("K6_42_116", prem, [argfact("Rem", 42, "80", 0), pair6("Ott", 116, "x80", 30)],
       "K6: witness [42, 116 pair-of-6tuples]")
K.emit("K6_61_114", prem, [argfact("Rem", 61, "81", 0), pair6("Ott", 114, "x81", 30)],
       "K6: witness [61, 114 pair-of-6tuples]")
K.emit("K6_42_100", prem, [argfact("Rem", 42, "82", 0), pair6("Ott", 100, "x82", 30)],
       "K6: [42, 100 pair-of-6tuples] denominator readout")

# TB4: [4-tuple stepped, Fbb 45] own-flip pins bonus(4): flip at 42+bonus+1
def tup4(name, flat, tag, off):
    a = pvars(3, off)
    fixed = len(name) + 2 + 1 + 3 * 5 + 1 + 2
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (flat, L1)
    elems = [longvar(L1, tag)] + a
    s = f"{name}( <{', '.join(elems)}> )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(<{', '.join(elems)}>)"
for s in range(43, 50):
    K.emit(f"TB4_{s}", prem, [tup4("Tf", s, f"{K.n:02d}", 0), argfact("Fbb", 45, f"x{K.n:02d}", 30)],
           f"TB4: [4-tuple {s}, Fbb 45] own-flip pins bonus(4)")
# TB4f: 4-tuple with quote + funcs (witness shape) own-flip
def tup4f(name, flat, tag, off):
    a = pvars(2, off)
    core = f"'cm', ff({a[0]}), ff({a[1]})"
    fixed = len(name) + 2 + 1 + len(core) + 2 + 1 + 2
    L1 = flat - fixed
    assert L1 >= 2 + len(tag), (flat, L1)
    s = f"{name}( <{core}, {longvar(L1, tag)}> )"
    assert len(s) == flat, (s, len(s), flat)
    return f"{name}(<{core}, {longvar(L1, tag)}>)"
for s in range(43, 50):
    K.emit(f"TB4f_{s}", prem, [tup4f("Tf", s, f"{K.n:02d}", 0), argfact("Fbb", 45, f"x{K.n:02d}", 30)],
           f"TB4f: [4-tuple with funcs {s}, Fbb 45] own-flip")
# WIT: witness [St-like stepped, Fr-like 13]: cell = name( ~-var, vars, 4-tuple, w1(x) )
for s in range(73, 80):
    a = pvars(3, K.n)
    core = f"<'commit', ff({a[0]}), ff({a[1]}), {a[2]}>"
    fixed = 6 + 5 + 2 + len(core) + 2 + 7 + 2
    L1 = s - fixed
    cell = f"St_I( ~id, {longvar(L1, f'{K.n:02d}')}, {core}, w1($zz) )"
    assert len(cell) == s, (cell, len(cell), s)
    body = (f"// WIT: [St-like {s}, Fr 13] witness trigger boundary\n"
            f"rule Init_W{s}:\n  [ Fr(~id) ]\n  -->\n  [ {cell.replace('( ', '(').replace(' )', ')')} ]\n\n"
            f"rule WIT_{s}:\n  [ {cell.replace('( ', '(').replace(' )', ')')}, Fr(~ni) ]\n"
            f"  --[ F{K.n}() ]->\n  [ Out(~ni) ]\n")
    lemma = f"lemma l_WIT_{s}:\n  exists-trace \"Ex #i. F{K.n}() @ #i\"\n"
    K.emit_raw(f"WIT_{s}", body, lemma)
K.write("probeK.spthy")

