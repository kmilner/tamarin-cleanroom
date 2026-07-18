#!/usr/bin/env python3
"""Round-12 miss inventory over bands9.tsv (20-field cells, round-11 dump).

Replicates the SHIPPED round-11 model (generate::group_widths_with, no
overrides) exactly, classifies every cell OK / FF / FW / FM, and clusters the
FM (family A) misses: delta from predicted L to the nearest observed band
edge, receiver shape, sibling profile, break direction, parity of the
predicted ribbon, and relief involvement.  Also scores L-mapping variants
(floor vs ceil lineLength) as a candidate second-order term.

Cell fields (band_dump round-11):
  0 flat  1 status  2 dtop 3 drec 4 nq 5 sqa 6 nargs 7 nfunc 8 nabbr
  9 ctup 10 bmax 11 tup_sur 12 uni_sur 13 cbmax 14 rec_sur 15 cnargs
  16 last_tup 17 rec5 18 rec7 19 rec9
"""
import sys, math, collections

def parse(path):
    rows = []
    for line in open(path):
        p = line.rstrip("\n").split("\t")
        if len(p) != 6:
            continue
        cells = []
        ok = True
        for cc in p[5].split("|"):
            seg = cc.split(":")
            if len(seg) != 20:
                ok = False
                break
            f = int(seg[0]); st = seg[1]
            (dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax,
             tup_sur, uni_sur, cbmax, rec_sur, cnargs, last_tup,
             rec5, rec7, rec9) = map(int, seg[2:])
            cells.append(dict(f=f, st=st, wrap=not st.startswith("F"),
                              sqa=sqa, nfunc=nfunc, nabbr=nabbr,
                              tup=tup_sur, uni=uni_sur, bonus=cbmax,
                              rec=rec_sur, rec7=rec7, last=last_tup,
                              nargs=cnargs, nq=nq))
        if ok and cells:
            rows.append((p[0], p[1], cells))
    return rows

def band_runs(st):
    if st == "NONE" or st.startswith("F"):
        return []
    return [tuple(map(int, r.split("-"))) for r in st.split(",")]

def hd(x):
    fl = math.floor(x)
    return int(fl if abs(x - fl - 0.5) < 1e-9 else math.floor(x + 0.5))

def shipped(cells):
    """Round-11 group_widths_with(cells, &[]) -> list of (kind, b, extras).
    kind: 'flat' (pred no wrap) | 'wrap' (pred wrap, b = fill ribbon).
    extras: dict with pass diagnostics."""
    n = len(cells)
    C = [c["f"] + c["rec"] for c in cells]
    Ctot = sum(C)
    out = []
    # pass 1
    budget1, wrap1 = [], []
    for i, c in enumerate(cells):
        bonus = c["bonus"] if c["last"] else 0
        b1 = 87 if n == 1 else max(87 + bonus - (Ctot - C[i]), 20)
        budget1.append(b1)
        wrap1.append(c["f"] > b1)
    # pass-1 fills
    fill1 = [None] * n
    for i, c in enumerate(cells):
        if not wrap1[i]:
            continue
        if n == 1:
            fill1[i] = 87
            continue
        num = float(c["f"] + c["rec7"] + c["nfunc"])
        t = num
        for j, cj in enumerate(cells):
            if j != i:
                w = 5.0 / 6.0 if (cj["sqa"] and (c["tup"] + c["uni"]) > 0) else 1.0
                t += w * C[j]
        b = hd(87.0 * num / t)
        b = max(20, min(b, max(c["f"] - 1, 20)))
        fill1[i] = b
    # pass 2 relief
    for i, c in enumerate(cells):
        if not wrap1[i]:
            out.append(("flat", max(budget1[i], c["f"]), dict(pass1=False)))
            continue
        saved = False
        b2 = None
        if n > 1:
            tot = 0
            for j, cj in enumerate(cells):
                if j != i:
                    truly = wrap1[j] and fill1[j] is not None and fill1[j] < cj["f"] - 2
                    tot += fill1[j] if truly else C[j]
            b2 = max(87 - tot, 20)
            if c["f"] <= b2:
                saved = True
        if saved:
            out.append(("flat", max(b2, c["f"]), dict(pass1=True, saved=True)))
        else:
            out.append(("wrap", fill1[i], dict(pass1=True, saved=False, b2=b2,
                                               b1=budget1[i])))
    return out

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "bands9.tsv")
    lmaps = {
        "floor(3b/2)": lambda b: 3 * b // 2,
        "ceil(3b/2)":  lambda b: (3 * b + 1) // 2,
    }
    cnt = {k: collections.Counter() for k in lmaps}
    # FM clustering under the SHIPPED floor map
    fm_delta = collections.Counter()      # signed L-delta to nearest band edge
    fm_bparity = collections.Counter()    # (b odd?, delta)
    fm_shape = collections.Counter()
    fm_relief = collections.Counter()
    fm_sibs = collections.Counter()
    fm_ex = collections.defaultdict(list)
    ff_shape = collections.Counter()
    ff_ex = []
    fw_shape = collections.Counter()
    fw_ex = []
    for fname, kind, cells in rows:
        n = len(cells)
        preds = shipped(cells)
        nwrap_obs = sum(1 for c in cells if c["wrap"])
        grp_abbr = sum(c["nabbr"] for c in cells)
        for i, c in enumerate(cells):
            pk, b, extras = preds[i]
            runs = band_runs(c["st"])
            for mname, lm in lmaps.items():
                L = lm(b)
                if c["wrap"] and pk == "flat":
                    cnt[mname]["FF"] += 1
                elif (not c["wrap"]) and pk == "wrap":
                    cnt[mname]["FW"] += 1
                elif c["wrap"]:
                    if c["st"] == "NONE":
                        cnt[mname]["NONE"] += 1
                    elif any(a <= L <= bb for a, bb in runs):
                        cnt[mname]["OK_fill"] += 1
                    else:
                        cnt[mname]["FM"] += 1
                else:
                    cnt[mname]["OK_flat"] += 1
            # detailed clustering under the shipped floor map
            L = 3 * b // 2
            if c["wrap"] and pk == "wrap" and c["st"] != "NONE" and runs \
               and not any(a <= L <= bb for a, bb in runs):
                best = None
                for a, bb in runs:
                    d = L - bb if L > bb else L - a
                    if best is None or abs(d) < abs(best):
                        best = d
                fm_delta[max(-25, min(25, best))] += 1
                fm_bparity[(b % 2, max(-4, min(4, best)))] += 1
                fm_shape[(c["tup"] > 0, c["uni"] > 0, c["nfunc"] > 0,
                          c["nabbr"] > 0, c["last"])] += 1
                fm_relief[(n, nwrap_obs, extras.get("saved"),
                           b == 20, best < 0)] += 1
                sibw = tuple(sorted((cj["f"], cj["wrap"]) for j, cj in
                                    enumerate(cells) if j != i))[:3]
                fm_sibs[(n, best if abs(best) <= 3 else (4 if best > 0 else -4))] += 1
                key = "small" if abs(best) <= 3 else "big"
                if len(fm_ex[key]) < 15:
                    fm_ex[key].append((fname, kind, n, i, c["f"], c["st"], b, L, best,
                                       [(cj["f"], cj["rec"], cj["wrap"]) for cj in cells]))
            if c["wrap"] and pk == "flat":
                ff_shape[(n, c["tup"] > 0, c["last"], c["nabbr"] > 0,
                          extras.get("saved", False))] += 1
                if len(ff_ex) < 20:
                    ff_ex.append((fname, kind, n, i, c["f"], c["st"], extras,
                                  [(cj["f"], cj["rec"], cj["wrap"]) for cj in cells]))
            if (not c["wrap"]) and pk == "wrap":
                fw_shape[(n, c["tup"] > 0, c["uni"] > 0, c["nq"] > 0,
                          c["nabbr"] > 0)] += 1
                if len(fw_ex) < 20:
                    fw_ex.append((fname, kind, n, i, c["f"], b, extras,
                                  [(cj["f"], cj["rec"], cj["wrap"]) for cj in cells]))
    for mname in lmaps:
        print(mname, dict(cnt[mname]))
    print("\nFM signed L-delta to nearest band edge (neg = predicted too small):")
    for k in sorted(fm_delta, key=lambda x: x):
        print("  ", k, fm_delta[k])
    print("\nFM (b parity, clamped delta):")
    for k, v in sorted(fm_bparity.items()):
        print("  ", k, v)
    print("\nFM shape (tup, uni, func, abbr, last_tup):")
    for k, v in fm_shape.most_common(12):
        print("  ", k, v)
    print("\nFM (n, nwrap_obs, saved, floor20, delta<0):")
    for k, v in fm_relief.most_common(15):
        print("  ", k, v)
    print("\nFM (n, delta clamp +-4):")
    for k, v in sorted(fm_sibs.items()):
        print("  ", k, v)
    print("\nFM small-delta examples:")
    for e in fm_ex["small"]:
        print("  ", e)
    print("\nFM big-delta examples:")
    for e in fm_ex["big"]:
        print("  ", e)
    print("\nFF shape (n, tup, last, abbr, relief-saved):")
    for k, v in ff_shape.most_common(15):
        print("  ", k, v)
    print("\nFF examples (file kind n i flat st extras row):")
    for e in ff_ex:
        print("  ", e)
    print("\nFW shape (n, tup, uni, nq, abbr):")
    for k, v in fw_shape.most_common(15):
        print("  ", k, v)
    print("\nFW examples (file kind n i flat pred_b extras row):")
    for e in fw_ex:
        print("  ", e)

if __name__ == "__main__":
    main()
