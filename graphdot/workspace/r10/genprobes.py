#!/usr/bin/env python3
# Round-10 probe generators. Four batteries, one .spthy each:
#   probeA.spthy (AuditSqaProbe)  — task 1: distinguish the two logged sqa models.
#     RA: [Faa(8 $vars)=45, Sib(s) sqa] s=40..48 step 1 (fills r9 parity gaps).
#         X wraps iff s > 42+k (k = sqa occupancy correction, C=flat-k);
#         Sib wraps iff s > 42+d (d = sqa own-width discount, eff=flat-d).
#     RB: [Faaa(8)=46, Sib(s)] s=39..46 (replication at a second base).
#     RC: order swap [Sib(s), Faa 45] s in {42,44}.
#     RD: triple [F(5)=28, H(5)=28, Sib(s)] s=30..35 (k,d in a 3-cell row).
#     RE: floor [Wf(13)=70, Sib(s)] s=20..24 (discount at the floor-20 budget).
#   probeB.spthy (FuncShapeProbe) — task 2: function-node corrections.
#     FB: [Faa 45, Q(func(...)) s] s=38..48 (func occupancy via X's flip; own flip).
#     FC: single-cell wide funcs (flat 106 / nested chain 112) — internal breaks.
#     FD: single-cell func fact flat 86..90 — the lone-cell boundary for funcs.
#     FE: wide func fact + siblings — fill layout of a squeezed func cell.
#   probeC.spthy (QuoteShapeProbe) — task 2: multi-quote corrections.
#     QA: [Faa 45, Qq('a..','b..') s] s=41..49 (two-quote fact own/occ).
#     QB: [Faa 45, Qm('a..', $ba) s] s=41..47 (one quote among 2 args).
#     QC: [Faa 45, Qt(<'a..', $ba, $bb>) s] s=38..46 (quote inside tuple-fact).
#   probeD.spthy (UnionShapeProbe) — task 3: ++-union display/breaks (multiset).
#     UA: single-cell wide unions (13/15/20 elems) — display format + breaks.
#     UB: single-cell union fact near the 87 boundary (one stretch-quote elem).
#     UC: unions nested in tuple / function argument.
#     UD: wide union fact + sibling — fill at squeezed budgets.
#     UE: [Faa 45, Un(union) s~39..46] — union-fact occupancy via X's flip.
#     DN: deep right-nested function chain, alone and squeezed.
import sys

def pvar(i, pref=""):
    a = "abcdefghijklmnopqrstuvwxyz"
    return "$" + pref + a[i // 26] + a[i % 26]

def pvars(n, off=0):
    return [pvar(i + off) for i in range(n)]

def argfact(name, n, off=0):
    return f"{name}({', '.join(pvars(n, off))})"

def sib(p, idx, name="Sib"):
    return f"{name}('{'a' * p}{idx}')"

def in_prem(k, off=100):
    return f"In(<{', '.join(pvars(k, off))}>)"

class Battery:
    def __init__(self, thy, builtins=None, functions=None):
        self.thy = thy
        self.builtins = builtins
        self.functions = functions
        self.rules, self.lemmas, self.names = [], [], []
        self.n = 0

    def emit(self, name, prems, concls, comment):
        self.rules.append(
            f"// {comment}\n"
            f"rule {name}:\n  [ {', '.join(prems)} ]\n  --[ F{self.n}() ]->\n  [ {', '.join(concls)} ]\n")
        self.lemmas.append(f"lemma l_{name}:\n  exists-trace \"Ex #i. F{self.n}() @ #i\"\n")
        self.names.append(name)
        self.n += 1

    def write(self, path):
        with open(path, "w") as f:
            f.write(f"theory {self.thy}\nbegin\n\n")
            if self.builtins:
                f.write(f"builtins: {self.builtins}\n\n")
            if self.functions:
                f.write(f"functions: {self.functions}\n\n")
            for r in self.rules:
                f.write(r + "\n")
            for l in self.lemmas:
                f.write(l + "\n")
            f.write("end\n")
        with open(path.replace(".spthy", ".names"), "w") as f:
            for c in self.names:
                f.write(c + "\n")
        print(f"wrote {path}: {len(self.rules)} rules")

# ---------------- Battery A: audit redo (sqa placement/magnitude) ----------------
A = Battery("AuditSqaProbe")
for s in range(40, 49):                      # RA: base Faa(8)=45
    A.emit(f"RA_{s:02d}", [in_prem(2)],
           [argfact("Faa", 8), sib(s - 11, f"{A.n:02d}")],
           f"RA: concl [Faa 45, Sib {s}] — X flips at s=42+k, sib at s=42+d")
for s in range(39, 47):                      # RB: base Faaa(8)=46
    A.emit(f"RB_{s:02d}", [in_prem(2)],
           [argfact("Faaa", 8), sib(s - 11, f"{A.n:02d}")],
           f"RB: concl [Faaa 46, Sib {s}] — X flips at s=41+k, sib at s=41+d")
for s in (42, 44):                           # RC: order swap
    A.emit(f"RC_{s:02d}", [in_prem(2)],
           [sib(s - 11, f"{A.n:02d}"), argfact("Faa", 8)],
           f"RC: concl [Sib {s}, Faa 45] — order-swapped RA")
for s in range(30, 36):                      # RD: triple, argfacts F(5)=H(5)=28
    A.emit(f"RD_{s:02d}", [in_prem(2)],
           [argfact("F", 5), argfact("H", 5, off=5), sib(s - 11, f"{A.n:02d}")],
           f"RD: concl [F 28, H 28, Sib {s}] — X flips at s=31+k, sib at s=31+d")
for s in range(20, 25):                      # RE: floor (sib budget 87-70=17 -> 20)
    A.emit(f"RE_{s:02d}", [in_prem(2)],
           [argfact("Wff", 13), sib(s - 11, f"{A.n:02d}")],
           f"RE: concl [Wff 70, Sib {s}] — sib at floor 20: wraps at s=21+d_floor")
A.write("probeA.spthy")

# ---------------- Battery B: function-node shapes ----------------
fn_decls = []
for ln in range(1, 9):
    fn_decls.append(f"{'u' * ln}/7")         # u..uuuuuuuu arity 7 (FB 41..48)
for ln in range(3, 6):
    fn_decls.append(f"{'v' * ln}/6")         # vvv..vvvvv arity 6 (FB 38..40)
for ln in range(1, 6):
    fn_decls.append(f"{'w' * ln}/16")        # w..wwwww arity 16 (FD 86..90)
fn_decls += ["q/20", "p/2"]
B = Battery("FuncShapeProbe", functions=", ".join(fn_decls))

def funcfact(fname, k, off=0, outer="Q"):
    return f"{outer}({fname}({', '.join(pvars(k, off))}))"

for s in range(38, 41):                      # FB: arity-6, name len s-35
    B.emit(f"FB_{s:02d}", [in_prem(2)],
           [argfact("Faa", 8), funcfact("v" * (s - 35), 6, off=8)],
           f"FB: concl [Faa 45, Q(func) {s}] — func occupancy via X flip at C>42")
for s in range(41, 49):                      # FB: arity-7, name len s-40
    B.emit(f"FB_{s:02d}", [in_prem(2)],
           [argfact("Faa", 8), funcfact("u" * (s - 40), 7, off=8)],
           f"FB: concl [Faa 45, Q(func) {s}]")
B.emit("FC_1", [in_prem(2)], [funcfact("q", 20)],
       "FC: lone Q(q(20 args)) flat 106 — does the reference break inside q(...)?")
chain = pvar(13)
for i in range(12, -1, -1):
    chain = f"p({pvar(i)}, {chain})"
B.emit("FC_3", [in_prem(2)], [f"Q({chain})"],
       "FC: lone Q(p($aa, p($ab, ...))) deep chain — internal break shape")
for ln in range(1, 6):                       # FD: lone func fact flat 85+len
    B.emit(f"FD_{85 + ln}", [in_prem(2)], [funcfact("w" * ln, 16)],
           f"FD: lone Q({'w' * ln}(16 args)) flat {85 + ln} — lone-cell boundary")
B.emit("FE_1", [in_prem(2)], [funcfact("q", 20), sib(19, f"{B.n:02d}")],
       "FE: [Q(q(20)) 106, Sib 30] — squeezed func fill")
B.emit("FE_2", [in_prem(2)], [funcfact("q", 20), argfact("Faa", 8, off=20)],
       "FE: [Q(q(20)) 106, Faa 45] — squeezed func fill vs argfact")
B.write("probeB.spthy")

# ---------------- Battery C: quote shapes ----------------
C = Battery("QuoteShapeProbe")
for s in range(41, 50):                      # QA: Qq('a'*q1+i, 'b'*4+i)
    q1 = s - 20                              # flat = 2+2+(q1+4)+2+8+2 = q1+20
    C.emit(f"QA_{s:02d}", [in_prem(2)],
           [argfact("Faa", 8),
            f"Qq('{'a' * q1}{C.n:02d}', '{'b' * 4}{C.n:02d}')"],
           f"QA: concl [Faa 45, Qq 2-quote {s}] — own flip 42+d2, occ flip 42+k2")
for s in range(41, 48):                      # QB: Qm('a'*q+i, $ba)
    q = s - 15                               # flat = 2+2+(q+4)+2+3+2 = q+15
    C.emit(f"QB_{s:02d}", [in_prem(2)],
           [argfact("Faa", 8), f"Qm('{'a' * q}{C.n:02d}', {pvar(8)})"],
           f"QB: concl [Faa 45, Qm quote+var {s}]")
for s in range(38, 47):                      # QC: Qt(<'a'*q+i, $ba, $bb>)
    q = s - 20                               # flat = 2+2+(1+(q+4)+2+3+2+3+1)+2 = q+20
    C.emit(f"QC_{s:02d}", [in_prem(2)],
           [argfact("Faa", 8), f"Qt(<'{'a' * q}{C.n:02d}', {pvar(8)}, {pvar(9)}>)"],
           f"QC: concl [Faa 45, Qt tuple-with-quote {s}] — quote-in-tuple corrections")
C.write("probeC.spthy")

# ---------------- Battery D: ++-unions + deep nesting ----------------
D = Battery("UnionShapeProbe", builtins="multiset")

def union(elems):
    return " ++ ".join(elems)

for n in (13, 15, 20):                       # UA: lone wide unions
    D.emit(f"UA_{n}", [in_prem(2)], [f"U({union(pvars(n))})"],
           f"UA: lone U({n}-elem union) — display format + break shape")
for q in range(37, 42):                      # UB: 8 vars + stretch quote
    D.emit(f"UB_{q}", [in_prem(2)],
           [f"U({union(pvars(8) + [chr(39) + 'z' * q + f'{D.n:02d}' + chr(39)])})"],
           f"UB: lone union with quote {q} — boundary sweep")
D.emit("UC_1", [in_prem(2)],
       [f"V(<{union(pvars(6))}, {pvar(30)}, '{'c' * 30}x1'>)"],
       "UC: union inside a tuple arg")
D.emit("UC_2", [in_prem(2)],
       [f"W(qq({union(pvars(14))}), {pvar(30)})"],
       "UC: union inside a function argument")
D.emit("UD_1", [in_prem(2)], [f"U({union(pvars(20))})", sib(19, f"{D.n:02d}")],
       "UD: [U(20-union), Sib 30] — squeezed union fill")
D.emit("UD_2", [in_prem(2)], [f"U({union(pvars(20))})", argfact("Faa", 8, off=20)],
       "UD: [U(20-union), Faa 45] — squeezed union fill vs argfact")
D.emit("DN_1", [in_prem(2)], [f"Q({chain})", sib(19, f"{D.n:02d}")],
       "DN: [deep chain, Sib 30] — squeezed nested-func fill")
# UE (union occupancy vs a 45-argfact) is generated in a SECOND pass
# (probeE.spthy) once UA/UB pin the union display format and hence the widths.
D.functions = "qq/1, p/2"
D.write("probeD.spthy")
