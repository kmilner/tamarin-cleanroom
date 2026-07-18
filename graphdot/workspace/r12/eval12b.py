#!/usr/bin/env python3
"""Round-12 FINAL law grid over 22-field band TSVs.

Axes:
  C:  rec | rec+ftup           (occupancy)
  s1: smax capped {99, 4, 3}   (pass-1 slack, any-arg)
  N:  f7 | f7+ftup             (fill numerator; nfunc dropped, NB refutes -nq)
  R:  q13 (min(hd(q+1/3),C)) | ceilq | hdq | t2 (shipped threshold)
  s2: 0 | e4 (1 if any top-level tuple/union arg >= 4 elems) | s1c1 (min(s1,1))
"""
import math, sys, glob, collections

def parse22(path):
    rows = []
    for line in open(path):
        p = line.rstrip("\n").split("\t")
        if len(p) != 6:
            continue
        cells = []
        ok = True
        for cc in p[5].split("|"):
            seg = cc.split(":")
            if len(seg) != 22:
                ok = False
                break
            f = int(seg[0]); st = seg[1]
            (dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax,
             tup_sur, uni_sur, cbmax, rec_sur, cnargs, last_tup,
             rec5, rec7, rec9, smax, ftup) = map(int, seg[2:])
            cells.append(dict(f=f, st=st, wrap=not st.startswith("F"),
                              sqa=sqa, nfunc=nfunc, nabbr=nabbr,
                              tup=tup_sur, uni=uni_sur, bonus=cbmax,
                              rec=rec_sur, rec7=rec7, last=last_tup,
                              nargs=cnargs, nq=nq, smax=smax, ftup=ftup))
        if ok and cells:
            rows.append((p[0], p[1], cells))
    return rows

def band_runs(st):
    if st == "NONE" or st.startswith("F"):
        return []
    return [tuple(map(int, r.split("-"))) for r in st.split(",")]

def in_band(st, L):
    if st.startswith("F"):
        return st != "FNONE"
    if st == "NONE":
        return False
    return any(a <= L <= b for a, b in band_runs(st))

def hd(x):
    fl = math.floor(x)
    return int(fl) if abs(x - fl - 0.5) < 1e-9 else int(math.floor(x + 0.5))

def run(rows, claw, s1cap, nlaw, rlaw, s2law, collect=None):
    ff = fw = fill_hit = fill_tot = 0
    for fname, kind, cells in rows:
        n = len(cells)
        C = [c["f"] + c["rec"] + (c["ftup"] if claw == "ft" else 0) for c in cells]
        S = sum(C)
        s1 = [min(c["smax"], s1cap) for c in cells]
        N = [c["f"] + c["rec7"] + (c["ftup"] if nlaw == "ft" else 0) for c in cells]
        pw, fills, qs = [], [None] * n, [None] * n
        for i, c in enumerate(cells):
            b_t = 87 if n == 1 else max(87 + s1[i] - (S - C[i]), 20)
            pw.append(c["f"] > b_t)
        for i, c in enumerate(cells):
            if not pw[i]:
                continue
            if n == 1:
                fills[i] = 87; qs[i] = 87.0
                continue
            t = float(N[i]) + sum(
                (5.0 / 6.0 if (cells[j]["sqa"] and (c["tup"] + c["uni"]) > 0) else 1.0) * C[j]
                for j in range(n) if j != i)
            qs[i] = 87.0 * N[i] / t
            fills[i] = max(20, min(hd(qs[i]), max(c["f"] - 1, 20)))
        final = list(pw)
        if n > 1:
            for i, c in enumerate(cells):
                if not pw[i]:
                    continue
                tot = 0
                for j, cj in enumerate(cells):
                    if j == i:
                        continue
                    if pw[j] and fills[j] is not None:
                        if rlaw == "t2":
                            tot += fills[j] if fills[j] < cj["f"] - 2 else C[j]
                        elif rlaw == "q13":
                            tot += min(hd(qs[j] + 1.0 / 3.0), C[j])
                        elif rlaw == "ceilq":
                            tot += min(int(math.ceil(qs[j] - 1e-9)), C[j])
                        elif rlaw == "hdq":
                            tot += min(hd(qs[j]), C[j])
                    else:
                        tot += C[j]
                s2 = 0
                if s2law == "e4":
                    s2 = 1 if c["bonus"] >= 4 else 0
                elif s2law == "s1c1":
                    s2 = min(s1[i], 1)
                if c["f"] <= max(87 + s2 - tot, 20):
                    final[i] = False
        for i, c in enumerate(cells):
            bad = None
            if final[i] and not c["wrap"]:
                fw += 1; bad = "FW"
            elif not final[i] and c["wrap"]:
                ff += 1; bad = "FF"
            elif c["wrap"]:
                fill_tot += 1
                if fills[i] is not None and in_band(c["st"], 3 * fills[i] // 2):
                    fill_hit += 1
                else:
                    bad = "FM"
            if bad and collect is not None:
                collect.append((fname, kind, i, c["f"], c["st"], bad,
                                fills[i], [(cj["f"], cj["rec"], cj["wrap"]) for cj in cells]))
    return ff, fw, fill_hit, fill_tot

def main():
    corpus = parse22("bands12.tsv")
    probes = []
    for p in sorted(glob.glob("b12_p*.tsv")):
        probes.extend(parse22(p))
    print(f"{'config':34s} {'corpFF':>6} {'FW':>5} {'fill':>15} | {'prbFF':>5} {'FW':>4} {'fill':>11}")
    results = []
    for claw in ("rec", "ft"):
        for s1cap in (99, 4, 3):
            for nlaw in ("f7", "ft"):
                for rlaw in ("q13", "ceilq", "hdq", "t2"):
                    for s2law in ("0", "e4", "s1c1"):
                        cf = run(corpus, claw, s1cap, nlaw, rlaw, s2law)
                        pf = run(probes, claw, s1cap, nlaw, rlaw, s2law)
                        tag = f"C={claw} s1c{s1cap} N={nlaw} R={rlaw} s2={s2law}"
                        results.append((cf[0] + cf[1] + (cf[3] - cf[2]) +
                                        3 * (pf[0] + pf[1] + pf[3] - pf[2]), tag, cf, pf))
    results.sort(key=lambda r: r[0])
    for score, tag, cf, pf in results[:20]:
        print(f"{tag:34s} {cf[0]:>6} {cf[1]:>5} {cf[2]:>7}/{cf[3]:<7} | "
              f"{pf[0]:>5} {pf[1]:>4} {pf[2]:>5}/{pf[3]:<5}  (score {score})")

if __name__ == "__main__":
    main()
