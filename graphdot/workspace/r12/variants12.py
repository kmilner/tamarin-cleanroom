#!/usr/bin/env python3
"""Round-12 law-variant competition over 20-field band TSVs.

Base = shipped round-11 model.  Axes:
  bonus:  ship (last-gated bmax) | zero
  N:      f7n = flat+rec7+nfunc (ship) | f7 = flat+rec7 | f7n-2q | f7-2q
  D fill sibling charge: C (flat+rec, ship) | flat | C7 (flat+rec7) | Nj
  relief sibling charge: t2 = fill if fill < flat-2 else C (ship)
                         t3 = fill if fill < flat-3 else C
                         t4 = fill if fill < flat-4 else C
                         allw = fill for every wrapping sib
Scores per config: corpus FF / FW / fill-in-band, probe same.
"""
import math, sys, glob, collections
sys.path.insert(0, '.')
from eval12 import parse, band_runs, hd

def in_band(st, L):
    if st.startswith("F"):
        return st != "FNONE"
    if st == "NONE":
        return False
    return any(a <= L <= b for a, b in band_runs(st))

def run(rows, blaw, nlaw, dlaw, rlaw):
    ff = fw = fill_hit = fill_tot = 0
    for fname, kind, cells in rows:
        n = len(cells)
        C = [c["f"] + c["rec"] for c in cells]
        S = sum(C)
        def nval(c):
            base = c["f"] + c["rec7"] + (c["nfunc"] if nlaw in ("f7n", "f7n-2q") else 0)
            if nlaw.endswith("-2q"):
                base -= 2 * c["nq"]
            return base
        N = [nval(c) for c in cells]
        def dcharge(j, i):
            cj = cells[j]
            if dlaw == "C":
                v = C[j]
            elif dlaw == "flat":
                v = cj["f"]
            elif dlaw == "C7":
                v = cj["f"] + cj["rec7"]
            else:  # Nj
                v = N[j]
            w = 5.0 / 6.0 if (cj["sqa"] and (cells[i]["tup"] + cells[i]["uni"]) > 0) else 1.0
            return w * v
        bon = [(c["bonus"] if (blaw == "ship" and c["last"]) else 0) for c in cells]
        pw, b1s = [], []
        for i, c in enumerate(cells):
            b_t = 87 if n == 1 else max(87 + bon[i] - (S - C[i]), 20)
            b1s.append(b_t)
            pw.append(c["f"] > b_t)
        fills = [None] * n
        for i, c in enumerate(cells):
            if not pw[i]:
                continue
            if n == 1:
                fills[i] = 87
                continue
            t = float(N[i]) + sum(dcharge(j, i) for j in range(n) if j != i)
            b = hd(87.0 * N[i] / t)
            fills[i] = max(20, min(b, max(c["f"] - 1, 20)))
        final_wrap = list(pw)
        if n > 1:
            for i, c in enumerate(cells):
                if not pw[i]:
                    continue
                tot = 0
                for j, cj in enumerate(cells):
                    if j == i:
                        continue
                    if pw[j] and fills[j] is not None:
                        if rlaw == "allw":
                            tot += fills[j]
                        else:
                            th = int(rlaw[1])  # t2/t3/t4
                            tot += fills[j] if fills[j] < cj["f"] - th else C[j]
                    else:
                        tot += C[j]
                if c["f"] <= max(87 - tot, 20):
                    final_wrap[i] = False
        for i, c in enumerate(cells):
            if final_wrap[i] and not c["wrap"]:
                fw += 1
            elif not final_wrap[i] and c["wrap"]:
                ff += 1
            elif c["wrap"]:
                fill_tot += 1
                if fills[i] is not None and in_band(c["st"], 3 * fills[i] // 2):
                    fill_hit += 1
    return ff, fw, fill_hit, fill_tot

def main():
    corpus = parse("../r11/bands9.tsv")
    probes = []
    for p in sorted(glob.glob("../r11/b9_*.tsv")):
        probes.extend(parse(p))
    print(f"{'config':32s} {'corpFF':>6} {'FW':>5} {'fill':>15} | {'prbFF':>5} {'FW':>4} {'fill':>11}")
    for blaw in ("ship", "zero"):
        for nlaw in ("f7n", "f7", "f7n-2q", "f7-2q"):
            for dlaw in ("C", "flat", "C7", "Nj"):
                for rlaw in ("t2", "t3", "t4", "allw"):
                    cf = run(corpus, blaw, nlaw, dlaw, rlaw)
                    pf = run(probes, blaw, nlaw, dlaw, rlaw)
                    tag = f"b={blaw} N={nlaw} D={dlaw} R={rlaw}"
                    print(f"{tag:32s} {cf[0]:>6} {cf[1]:>5} {cf[2]:>7}/{cf[3]:<7} | "
                          f"{pf[0]:>5} {pf[1]:>4} {pf[2]:>5}/{pf[3]:<5}")

if __name__ == "__main__":
    main()
