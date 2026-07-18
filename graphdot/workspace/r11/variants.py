#!/usr/bin/env python3
"""Final round-11 law selection over 16-field band TSVs (post-hang engine).

Axes:
  C law:      old (flat+tup+uni)  |  rec (flat+rec_sur)
  N law:      old (flat+uni+nfunc)  |  rec (flat+rec_sur+nfunc)
  bonus law:  ship (bmax any fact)  |  single (bmax iff nargs==1)  |  zero
  relief:     off | on  (pass-2: wrapping sibs charged at their pass-1 fill
              alloc, flat sibs at C; no bonus; floor 20)
Fill rounding fixed at half-down; sqa-sib 5/6 for tuple receivers kept.

Scores per config: trigger errors (FF+FW) and fill in-band hits, on
(a) corpus bands7, (b) all probe TSVs.
"""
import math, sys, glob
sys.path.insert(0, '.')
from eval import in_band

def parse16(path):
    rows = []
    for line in open(path):
        p = line.rstrip("\n").split("\t")
        if len(p) != 6:
            continue
        cells = []
        ok = True
        for cc in p[5].split("|"):
            seg = cc.split(":")
            if len(seg) != 16:
                ok = False
                break
            f = int(seg[0]); st = seg[1]
            (dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax,
             tup, uni, cbmax, rec, cnargs) = map(int, seg[2:])
            cells.append(dict(f=f, st=st, wrap=not st.startswith("F"), sqa=sqa,
                              nfunc=nfunc, nabbr=nabbr, tup=tup, uni=uni,
                              bonus=cbmax, rec=rec, nargs=cnargs))
        if ok and cells:
            rows.append((p[0], cells))
    return rows

def hd(x):
    fl = math.floor(x)
    return int(fl) if abs(x - fl - 0.5) < 1e-9 else int(math.floor(x + 0.5))

def run(rows, claw, nlaw, blaw, relief):
    ff = fw = fill_hit = fill_tot = flat_ok = 0
    for fname, cells in rows:
        n = len(cells)
        C = [(c["f"] + (c["rec"] if claw == "rec" else c["tup"] + c["uni"])) for c in cells]
        S = sum(C)
        N = [(c["f"] + (c["rec"] if nlaw == "rec" else c["uni"]) + c["nfunc"]) for c in cells]
        bon = [(0 if blaw == "zero" else (c["bonus"] if (blaw == "ship" or c["nargs"] == 1) else 0))
               for c in cells]
        # pass 1
        pw = []
        for i, c in enumerate(cells):
            b_t = 87 if n == 1 else max(87 + bon[i] - (S - C[i]), 20)
            pw.append(c["f"] > b_t)
        # pass-1 fills
        fills = [None] * n
        for i, c in enumerate(cells):
            if not pw[i]:
                continue
            if n == 1:
                fills[i] = 87
                continue
            t = float(N[i])
            for j, cj in enumerate(cells):
                if j != i:
                    w = 5.0 / 6.0 if (cj["sqa"] and (cells[i]["tup"] or cells[i]["rec"]) > 0) else 1.0
                    t += w * C[j]
            b = hd(87.0 * N[i] / t)
            fills[i] = max(20, min(b, max(c["f"] - 1, 20)))
        # pass 2: relief
        final_wrap = list(pw)
        if relief and n > 1:
            for i, c in enumerate(cells):
                if not pw[i]:
                    continue
                tot = 0
                for j in range(n):
                    if j != i:
                        tot += fills[j] if pw[j] else C[j]
                if c["f"] <= max(87 - tot, 20):
                    final_wrap[i] = False
        for i, c in enumerate(cells):
            if final_wrap[i] and not c["wrap"]:
                fw += 1
            elif not final_wrap[i] and c["wrap"]:
                ff += 1
            elif c["wrap"]:
                fill_tot += 1
                if in_band(c["st"], 3 * fills[i] // 2):
                    fill_hit += 1
            else:
                flat_ok += 1
    return ff, fw, fill_hit, fill_tot

def main():
    corpus = parse16("bands7.tsv")
    probes = []
    for p in ["bandsGHI7.tsv", "bandsK7.tsv"] + sorted(glob.glob("b7_*.tsv")):
        probes.extend(parse16(p))
    print(f"{'config':26s} {'corpus FF':>9} {'FW':>5} {'fill':>15} | {'probe FF':>8} {'FW':>4} {'fill':>11}")
    for claw in ("old", "rec"):
        for nlaw in ("old", "rec"):
            for blaw in ("ship", "single", "zero"):
                for relief in (False, True):
                    cf = run(corpus, claw, nlaw, blaw, relief)
                    pf = run(probes, claw, nlaw, blaw, relief)
                    tag = f"C={claw} N={nlaw} b={blaw}{' R' if relief else ''}"
                    print(f"{tag:26s} {cf[0]:>9} {cf[1]:>5} {cf[2]:>7}/{cf[3]:<7} | "
                          f"{pf[0]:>8} {pf[1]:>4} {pf[2]:>5}/{pf[3]:<5}")

if __name__ == "__main__":
    main()
