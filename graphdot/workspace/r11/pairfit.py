#!/usr/bin/env python3
"""Fit candidate fill-allocation pair-models on corpus 2-cell rows where BOTH
cells wrap (the round-10 under-sampled interaction), and 2-cell rows with one
wrapping cell (control).  Score = rows where every wrapping cell's predicted L
(=3b//2) lies inside its observed band.  Restricted to rows with no
abbreviation-name tokens (nabbr==0 for all cells) so display width == internal
width (mostly)."""
import sys, math, collections
from eval import parse, band_runs, in_band

def ribbon_ok(c, b):
    return in_band(c["st"], 3 * b // 2)

def shipped_fill(cells, i, C):
    c = cells[i]
    num = float(c["f"] + c["uni"] + c["nfunc"])
    t = num
    for j, cj in enumerate(cells):
        if j != i:
            w = 5.0/6.0 if (cj["sqa"] and c["tup"] > 0) else 1.0
            t += w * C[j]
    b = math.floor(87.0 * num / t + 0.5)
    return max(20, min(b, max(c["f"] - 1, 20)))

def clamp(b, f):
    return max(20, min(b, max(f - 1, 20)))

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "bands6.tsv")
    # collect 2-cell rows, all cells nabbr==0, both wrapping
    both, onewrap = [], []
    for fname, kind, cells in rows:
        if len(cells) != 2 or any(c["nabbr"] for c in cells):
            continue
        w = [c["wrap"] for c in cells]
        if all(w):
            both.append((fname, cells))
        elif any(w):
            onewrap.append((fname, cells))
    print(f"2-cell abbr-free rows: both-wrap {len(both)}, one-wrap {len(onewrap)}")

    def C_of(cells):
        return [c["f"] + c["tup"] + c["uni"] for c in cells]
    def N_of(c):
        return c["f"] + c["uni"] + c["nfunc"]

    models = {}
    # M0 shipped proportional
    def m0(cells):
        C = C_of(cells)
        return [shipped_fill(cells, i, C) for i in range(2)]
    models["M0 shipped prop"] = m0
    # M1: post-wrap-charge refinement: b_i = 87 - prop_j (both wrap)
    def m1(cells):
        C = C_of(cells)
        p = [shipped_fill(cells, i, C) for i in range(2)]
        return [clamp(87 - p[1], cells[0]["f"]), clamp(87 - p[0], cells[1]["f"])]
    models["M1 87-prop_other"] = m1
    # M2: narrow gets floor 20, wide gets 87-20=67
    def m2(cells):
        i_n = 0 if cells[0]["f"] <= cells[1]["f"] else 1
        out = [0, 0]
        out[i_n] = 20
        out[1 - i_n] = clamp(67, cells[1 - i_n]["f"])
        out[i_n] = clamp(20, cells[i_n]["f"])
        return out
    models["M2 floor/67"] = m2
    # M3: narrow = max(87 - f_wide, 20); wide = 87 - that
    def m3(cells):
        i_n = 0 if cells[0]["f"] <= cells[1]["f"] else 1
        x = max(87 - cells[1 - i_n]["f"], 20)
        out = [0, 0]
        out[i_n] = clamp(x, cells[i_n]["f"])
        out[1 - i_n] = clamp(87 - out[i_n], cells[1 - i_n]["f"])
        return out
    models["M3 subtractive-nf"] = m3
    # M4: prop over flats (no occupancy, no numerator terms)
    def m4(cells):
        t = cells[0]["f"] + cells[1]["f"]
        return [clamp(math.floor(87.0 * c["f"] / t + 0.5), c["f"]) for c in cells]
    models["M4 prop flats"] = m4
    # M5: prop over N (both numerators)
    def m5(cells):
        t = N_of(cells[0]) + N_of(cells[1])
        return [clamp(math.floor(87.0 * N_of(c) / t + 0.5), c["f"]) for c in cells]
    models["M5 prop N/N"] = m5
    # M6: narrow = prop, wide = 87 - narrow-prop
    def m6(cells):
        C = C_of(cells)
        i_n = 0 if cells[0]["f"] <= cells[1]["f"] else 1
        p_n = shipped_fill(cells, i_n, C)
        out = [0, 0]
        out[i_n] = p_n
        out[1 - i_n] = clamp(87 - p_n, cells[1 - i_n]["f"])
        return out
    models["M6 prop-narrow,rest-wide"] = m6
    # M7: wide = prop, narrow = 87 - wide-prop
    def m7(cells):
        C = C_of(cells)
        i_n = 0 if cells[0]["f"] <= cells[1]["f"] else 1
        p_w = shipped_fill(cells, 1 - i_n, C)
        out = [0, 0]
        out[1 - i_n] = p_w
        out[i_n] = clamp(87 - p_w, cells[i_n]["f"])
        return out
    models["M7 prop-wide,rest-narrow"] = m7

    for name, fn in models.items():
        hit_rows = hit_cells = tot_cells = 0
        for fname, cells in both:
            bs = fn(cells)
            oks = [ribbon_ok(c, b) for c, b in zip(cells, bs)]
            tot_cells += 2
            hit_cells += sum(oks)
            hit_rows += all(oks)
        # one-wrap control: score the wrapping cell only
        c_hit = c_tot = 0
        for fname, cells in onewrap:
            bs = fn(cells)
            for c, b in zip(cells, bs):
                if c["wrap"]:
                    c_tot += 1
                    c_hit += ribbon_ok(c, b)
        print(f"{name:26s} both-wrap rows {hit_rows}/{len(both)} cells {hit_cells}/{tot_cells}"
              f" | one-wrap cell {c_hit}/{c_tot}")

    # dump the both-wrap inventory for eyeballing: flats + observed ribbon ranges
    if "-v" in sys.argv:
        def rr(st):
            rs = band_runs(st)
            if not rs:
                return "NONE"
            def l2r(L):
                v = L / 1.5
                fl = math.floor(v)
                if abs(v - fl - 0.5) < 1e-9:
                    f = int(fl)
                    return f if f % 2 == 0 else f + 1
                return round(v)
            return ",".join(f"{l2r(a)}..{l2r(b)}" for a, b in rs)
        for fname, cells in both[:60]:
            print(" ", fname, [(c["f"], c["tup"], c["uni"], c["nfunc"], rr(c["st"])) for c in cells])

if __name__ == "__main__":
    main()
