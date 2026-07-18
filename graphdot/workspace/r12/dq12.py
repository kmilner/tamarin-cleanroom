#!/usr/bin/env python3
"""Round-12: needed-Δq analysis. For every banded wrap-predicted cell compute
the allowed ribbon interval B_ok = {b : 3b//2 in band} (union), q, and the
signed correction dq needed to land hd(q) in B_ok (0 if already OK).
Group mean/quantiles of dq by shape features to identify the biased term.
"""
import sys, math, collections
from eval12 import parse, band_runs, hd
from frac12 import shipped_ctx, clamp_b

def allowed_bs(runs, f):
    ok = set()
    for b in range(20, max(f, 90) + 2):
        L = 3 * b // 2
        if any(a <= L <= bb for a, bb in runs):
            ok.add(b)
    return ok

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "../r11/bands9.tsv")
    # per-feature accumulation: feature -> [sum_dq, n, n_pos, n_neg]
    feats = collections.defaultdict(lambda: [0.0, 0, 0, 0])
    def acc(name, val, dq):
        s = feats[(name, val)]
        s[0] += dq; s[1] += 1
        if dq > 0: s[2] += 1
        if dq < 0: s[3] += 1
    for fname, kind, cells in rows:
        wrap1, qs, relief = shipped_ctx(cells)
        n = len(cells)
        for i, c in enumerate(cells):
            if not c["wrap"] or c["st"] == "NONE" or not wrap1[i]:
                continue
            if relief[i] is not None and c["f"] <= relief[i]:
                continue
            runs = band_runs(c["st"])
            if not runs:
                continue
            q = qs[i]
            b = clamp_b(hd(q), c["f"])
            if b != hd(q):
                continue  # clamped cells are insensitive to q
            ok = allowed_bs(runs, c["f"])
            ok = {x for x in ok if 20 <= x <= max(c["f"] - 1, 20)}
            if not ok:
                continue
            if b in ok:
                dq = 0.0
            else:
                # distance from q to the nearest q' with hd(q') in ok
                cands = []
                for t in ok:
                    # q' range mapping to t under hd: [t-0.5, t+0.5] (with .5 down)
                    lo, hi = t - 0.5, t + 0.5
                    if q < lo:
                        cands.append(lo - q + 1e-6)
                    elif q > hi:
                        cands.append(-(q - hi) - 1e-6)
                    else:
                        cands.append(0.0)
                dq = min(cands, key=abs)
            sibs = [cj for j, cj in enumerate(cells) if j != i]
            sib_rec = sum(cj["rec"] for cj in sibs)
            sib_rec7 = sum(cj["rec7"] for cj in sibs)
            sib_wrapping = sum(1 for cj in sibs if cj["wrap"])
            sib_sqa = sum(1 for cj in sibs if cj["sqa"])
            acc("nfunc", min(c["nfunc"], 4), dq)
            acc("rec7", min(c["rec7"], 8), dq)
            acc("rec_minus_rec7", min(c["rec"] - c["rec7"], 6), dq)
            acc("tup", c["tup"] > 0, dq)
            acc("uni", c["uni"] > 0, dq)
            acc("nabbr", min(c["nabbr"], 4), dq)
            acc("last", c["last"], dq)
            acc("bonus_last", c["bonus"] if c["last"] else 0, dq)
            acc("n", min(n, 6), dq)
            acc("nwrapobs", (min(n, 4), min(sib_wrapping, 3)), dq)
            acc("sib_rec", min(sib_rec, 10), dq)
            acc("sib_rec_gap", min(sib_rec - sib_rec7, 6), dq)
            acc("sib_sqa", min(sib_sqa, 2), dq)
            acc("nq", min(c["nq"], 4), dq)
            acc("nargs", min(c["nargs"], 8), dq)
    for (name, val), (s, cnt, npos, nneg) in sorted(feats.items()):
        print(f"{name:14} {str(val):10} n={cnt:6} mean_dq={s/max(cnt,1):+.4f} pos={npos:5} ({100.0*npos/max(cnt,1):4.1f}%) neg={nneg:5} ({100.0*nneg/max(cnt,1):4.1f}%)")

if __name__ == "__main__":
    main()
