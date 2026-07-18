#!/usr/bin/env python3
"""Round-12: diagnose the +1/-1 FM band-edge families.

For every wrapping predicted-wrap cell (shipped round-11 model), compute the
raw proportional quotient q = 87*N/t, its fractional part, the predicted
ribbon b = hd(q), L = 3b//2, and the position of L in the observed band.
Histogram frac for: OK cells, FM delta=+1, FM delta=-1, other FM.
Also: OK-cell margin to band edges (would a global -1 L shift break them?),
and re-score candidate variants:
  A  b = hd(q)          L = 3b//2          (shipped)
  B  b = floor(q)       L = 3b//2
  C  b = hd(q)          L = 3b//2 - 1
  D  b = hd(q) - 1      L = 3b//2
  E  b = hd(q)          L = (3b-1)//2      (even-b down 1)
  F  b = floor(q + 1/3) L = 3b//2
  G  q = 87*N/t with t = N + sum C_j + 1   hd, floor L
  H  q = (87*N - t/2)/t  (= 87N/t - 1/2)   hd == floor(87N/t) identical to B
  I  b = hd(86.5*N/t)   L = 3b//2
"""
import sys, math, collections
from eval12 import parse, band_runs, hd

def shipped_ctx(cells):
    n = len(cells)
    C = [c["f"] + c["rec"] for c in cells]
    Ctot = sum(C)
    budget1, wrap1 = [], []
    for i, c in enumerate(cells):
        bonus = c["bonus"] if c["last"] else 0
        b1 = 87 if n == 1 else max(87 + bonus - (Ctot - C[i]), 20)
        budget1.append(b1)
        wrap1.append(c["f"] > b1)
    qs = [None] * n
    fill1 = [None] * n
    for i, c in enumerate(cells):
        if not wrap1[i]:
            continue
        if n == 1:
            fill1[i] = 87.0
            qs[i] = 87.0
            continue
        num = float(c["f"] + c["rec7"] + c["nfunc"])
        t = num
        for j, cj in enumerate(cells):
            if j != i:
                w = 5.0 / 6.0 if (cj["sqa"] and (c["tup"] + c["uni"]) > 0) else 1.0
                t += w * C[j]
        qs[i] = 87.0 * num / t
        fill1[i] = float(max(20, min(hd(qs[i]), max(c["f"] - 1, 20))))
    relief = [None] * n
    for i, c in enumerate(cells):
        if not wrap1[i]:
            continue
        if n > 1:
            tot = 0.0
            for j, cj in enumerate(cells):
                if j != i:
                    truly = wrap1[j] and fill1[j] is not None and fill1[j] < cj["f"] - 2
                    tot += fill1[j] if truly else C[j]
            relief[i] = max(87.0 - tot, 20.0)
    return wrap1, qs, relief

def clamp_b(b, f):
    return max(20, min(b, max(f - 1, 20)))

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "../r11/bands9.tsv")
    frac_hist = {k: collections.Counter() for k in ("OK", "FM+1", "FM-1", "FMother")}
    margin_lo = collections.Counter()
    margin_hi = collections.Counter()
    variants = {
        "A hd,floorL":    lambda q, f: 3 * clamp_b(hd(q), f) // 2,
        "B floor,floorL": lambda q, f: 3 * clamp_b(math.floor(q), f) // 2,
        "C hd,L-1":       lambda q, f: 3 * clamp_b(hd(q), f) // 2 - 1,
        "D hd-1,floorL":  lambda q, f: 3 * clamp_b(hd(q) - 1, f) // 2,
        "E hd,(3b-1)//2": lambda q, f: (3 * clamp_b(hd(q), f) - 1) // 2,
        "F floor(q+1/3)": lambda q, f: 3 * clamp_b(math.floor(q + 1.0 / 3.0), f) // 2,
        "I hd(q*86.5/87)":lambda q, f: 3 * clamp_b(hd(q * 86.5 / 87.0), f) // 2,
    }
    vcnt = {k: collections.Counter() for k in variants}
    for fname, kind, cells in rows:
        wrap1, qs, relief = shipped_ctx(cells)
        for i, c in enumerate(cells):
            if not c["wrap"] or c["st"] == "NONE":
                continue
            if not wrap1[i]:
                continue  # FF handled elsewhere
            if relief[i] is not None and c["f"] <= relief[i]:
                continue  # relief-saved (pred flat): FF cell
            q = qs[i]
            runs = band_runs(c["st"])
            if not runs:
                continue
            b = clamp_b(hd(q), c["f"])
            L = 3 * b // 2
            inb = any(a <= L <= bb for a, bb in runs)
            clamped = (b != hd(q))
            if inb:
                key = "OK"
                best = 0
                lo_m = min(L - a for a, bb in runs if a <= L <= bb)
                hi_m = min(bb - L for a, bb in runs if a <= L <= bb)
                margin_lo[min(lo_m, 6)] += 1
                margin_hi[min(hi_m, 6)] += 1
            else:
                best = None
                for a, bb in runs:
                    d = L - bb if L > bb else L - a
                    if best is None or abs(d) < abs(best):
                        best = d
                key = "FM+1" if best == 1 else ("FM-1" if best == -1 else "FMother")
            fr = q - math.floor(q)
            bucket = ("CLAMP" if clamped else round(math.floor(fr * 10) / 10.0, 1))
            frac_hist[key][bucket] += 1
            for vn, vf in variants.items():
                Lv = vf(q, c["f"])
                vcnt[vn]["ok" if any(a <= Lv <= bb for a, bb in runs) else "miss"] += 1
    print("frac histogram (floor(10*frac)/10; CLAMP = clamped to floor/flat-1):")
    for key in ("OK", "FM+1", "FM-1", "FMother"):
        tot = sum(frac_hist[key].values())
        print(f"  {key} (n={tot}):")
        for bucket in ["CLAMP"] + [round(x / 10.0, 1) for x in range(10)]:
            v = frac_hist[key].get(bucket, 0)
            if v:
                print(f"    {bucket}: {v} ({100.0*v/max(tot,1):.1f}%)")
    print("\nOK margin to band lo (clamped 6):", dict(sorted(margin_lo.items())))
    print("OK margin to band hi (clamped 6):", dict(sorted(margin_hi.items())))
    print("\nvariant scores (in-band / miss over wrap-predicted banded cells):")
    for vn in variants:
        print(f"  {vn}: {dict(vcnt[vn])}")

if __name__ == "__main__":
    main()
