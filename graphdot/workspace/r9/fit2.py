#!/usr/bin/env python3
"""Fit trigger + fill layers with shape-corrected occupancies C.

bands2 format per cell: flat:STATUS:dtop:drec:nq:sqa:nargs
Trigger model: cell wraps iff flat > max(W - sum_{j!=i} C_j, FLOOR)
  C_j = flat_j + at*dtop_j (or drec) + aq*nq_j + asq*sqa_j
Fill model candidates for wrapping cells (budget in ribbon space -> L=3b//2):
  resid   = trigger budget
  propF   = round-family(W * flat_i / sum flat_j)
  propC   = W * C_i-ish / sum C_j ...
  maxRP   = max(resid, propF)
"""
import sys, math
from collections import Counter

def rib_to_L(b):
    return (3 * int(b)) // 2

def parse(path):
    groups = []
    with open(path) as f:
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) != 6:
                continue
            cells = []
            okrow = True
            for c in p[5].split("|"):
                seg = c.split(":")
                if len(seg) != 7:
                    okrow = False
                    break
                flat = int(seg[0]); status = seg[1]
                dtop, drec, nq, sqa, nargs = map(int, seg[2:])
                if status.startswith("F"):
                    lo = None if status in ("FNONE",) else int(status[1:])
                    cells.append(dict(f=flat, kind="fit", lo=lo, dtop=dtop,
                                      drec=drec, nq=nq, sqa=sqa, na=nargs))
                elif status == "NONE":
                    cells.append(dict(f=flat, kind="wrap", runs=None, dtop=dtop,
                                      drec=drec, nq=nq, sqa=sqa, na=nargs))
                else:
                    runs = [tuple(map(int, r.split("-"))) for r in status.split(",")]
                    cells.append(dict(f=flat, kind="wrap", runs=runs, dtop=dtop,
                                      drec=drec, nq=nq, sqa=sqa, na=nargs))
            if okrow and cells:
                groups.append(dict(file=p[0], kind=p[1], cells=cells))
    return groups

def C_of(c, at, aq, asq, use_rec=False):
    d = c["drec"] if use_rec else c["dtop"]
    return c["f"] + at * d + aq * c["nq"] + asq * c["sqa"]

def trigger_eval(groups, at, aq, asq, W=87, floor=20, use_rec=False):
    """Return (wrong_wrap_pred, wrong_fit_pred, total)."""
    fp = fn = tot = 0
    for g in groups:
        cs = g["cells"]
        if len(cs) < 2:
            continue
        Cs = [C_of(c, at, aq, asq, use_rec) for c in cs]
        S = sum(Cs)
        for c, Ci in zip(cs, Cs):
            b = max(W - (S - Ci), floor)
            pred_wrap = c["f"] > b
            obs_wrap = c["kind"] == "wrap"
            tot += 1
            if pred_wrap and not obs_wrap:
                fp += 1
            elif not pred_wrap and obs_wrap:
                fn += 1
    return fp, fn, tot

def in_band(c, L):
    if c["kind"] == "fit":
        return c["lo"] is not None and L >= c["lo"]
    if c["runs"] is None:
        return False
    return any(a <= L <= b for a, b in c["runs"])

def fill_eval(groups, fill_fn, at, aq, asq, W=87, floor=20, use_rec=False):
    """fill_fn(flats, Cs, i, trig_b) -> ribbon budget for wrapping cell i."""
    hit = tot = none = fitviol = fittot = 0
    for g in groups:
        cs = g["cells"]
        if len(cs) < 2:
            continue
        Cs = [C_of(c, at, aq, asq, use_rec) for c in cs]
        S = sum(Cs)
        flats = [c["f"] for c in cs]
        for i, c in enumerate(cs):
            trig_b = max(87 - (S - Cs[i]), floor)
            b = fill_fn(flats, Cs, i, trig_b)
            L = rib_to_L(b)
            if c["kind"] == "wrap":
                if c["runs"] is None:
                    none += 1
                    continue
                tot += 1
                if in_band(c, L):
                    hit += 1
            else:
                fittot += 1
                if not in_band(c, L):
                    fitviol += 1
    return hit, tot, none, fitviol, fittot

def main():
    paths = sys.argv[1:]
    groups = []
    for p in paths:
        groups += parse(p)
    multi = [g for g in groups if len(g["cells"]) >= 2]
    print(f"multi-cell groups: {len(multi)}")

    print("== trigger grid (fp = pred wrap, obs fit; fn = pred fit, obs wrap)")
    for (at, aq, asq, rec) in [
        (0, 0, 0, False),        # plain flat-sum
        (2, 0, -4, False),       # tuple 2n-4 top-level, sqa -4
        (2, 0, -4, True),        # recursive tuples
        (2, -2, 0, False),       # per-quote -2 instead of sqa
        (2, -2, -2, False),
        (2, 0, -2, False),
        (1, 0, -4, False),
        (3, 0, -4, False),
        (2, -1, -3, False),
    ]:
        fp, fn, tot = trigger_eval(multi, at, aq, asq, use_rec=rec)
        print(f"  at={at} aq={aq} asq={asq} rec={int(rec)}: fp {fp}  fn {fn}  err {100*(fp+fn)/max(tot,1):.3f}% of {tot}")

    print("== fill candidates (band-hit on wrapping cells; trigger at=2,asq=-4)")
    at, aq, asq = 2, 0, -4
    def resid(fl, Cs, i, tb):
        return tb
    def propF(fl, Cs, i, tb):
        T = sum(fl)
        return max(int(math.floor(87 * fl[i] / T)), 20)
    def propF_half(fl, Cs, i, tb):
        T = sum(fl)
        return max(int(math.floor(87 * fl[i] / T + 0.5)), 20)
    def propC(fl, Cs, i, tb):
        T = sum(Cs)
        return max(int(math.floor(87 * Cs[i] / T + 0.5)), 20)
    def maxRP(fl, Cs, i, tb):
        return max(tb, propF_half(fl, Cs, i, tb))
    def maxRPf(fl, Cs, i, tb):
        return max(tb, propF(fl, Cs, i, tb))
    def propFsibC(fl, Cs, i, tb):
        T = fl[i] + sum(Cs) - Cs[i]
        return max(int(math.floor(87 * fl[i] / T + 0.5)), 20)
    def maxR_propFsibC(fl, Cs, i, tb):
        return max(tb, propFsibC(fl, Cs, i, tb))
    for name, fn_ in [("resid", resid), ("propF_floor", propF),
                      ("propF_half", propF_half), ("propC", propC),
                      ("max(resid,propF_half)", maxRP),
                      ("max(resid,propF_floor)", maxRPf),
                      ("propF_sibC", propFsibC),
                      ("max(resid,propF_sibC)", maxR_propFsibC)]:
        hit, tot, none, fv, ft = fill_eval(multi, fn_, at, aq, asq)
        print(f"  {name:24s}: band-hit {hit}/{tot} = {100*hit/max(tot,1):.2f}%  (NONE {none})  fit-viol {fv}/{ft}")

if __name__ == "__main__":
    main()
