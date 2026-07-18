#!/usr/bin/env python3
"""Round-11 miss inventory over bands6.tsv (14-field cells).

Replicates the shipped round-10 model (generate::group_widths) exactly and
classifies every cell: OK / FF (false flat: obs wrap, pred fit) / FW (false
wrap: obs flat, pred wrap) / FM (both wrap, predicted L outside the observed
band).  Aggregates by group abbreviation content and shape for family
clustering (open-side round-10 integration families 1-4).
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
            if len(seg) != 14:
                ok = False
                break
            f = int(seg[0]); st = seg[1]
            (dtop, drec, nq, sqa, nargs, nfunc, nabbr, ctup, bmax,
             tup, uni, cbmax) = map(int, seg[2:])
            cells.append(dict(f=f, st=st, wrap=not st.startswith("F"),
                              sqa=sqa, nfunc=nfunc, nabbr=nabbr,
                              tup=tup, uni=uni, bonus=cbmax))
        if ok and cells:
            rows.append((p[0], p[1], cells))
    return rows

def band_runs(st):
    """Observed-valid L set for a wrapping cell: list of (lo,hi); [] if NONE."""
    if st == "NONE":
        return []
    return [tuple(map(int, r.split("-"))) for r in st.split(",")]

def in_band(st, L):
    if st.startswith("F"):
        return st != "FNONE"
    return any(a <= L <= b for a, b in band_runs(st))

def shipped_widths(cells):
    """Exact mirror of generate::group_widths_with(cells, &[])."""
    n = len(cells)
    C = [c["f"] + c["tup"] + c["uni"] for c in cells]
    S = sum(C)
    out = []
    for i, c in enumerate(cells):
        f = c["f"]
        if n == 1:
            # a lone cell is laid out at 87 and wraps iff its flat exceeds it
            out.append((87, f > 87))
            continue
        b_t = max(87 + c["bonus"] - (S - C[i]), 20)
        if f <= b_t:
            out.append((max(b_t, f), False))
            continue
        num = float(f + c["uni"] + c["nfunc"])
        t = num
        for j, cj in enumerate(cells):
            if j != i:
                w = 5.0/6.0 if (cj["sqa"] and c["tup"] > 0) else 1.0
                t += w * C[j]
        b = math.floor(87.0 * num / t + 0.5)
        b = max(20, min(b, max(f - 1, 20)))
        out.append((b, True))
    return out

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "bands6.tsv")
    cnt = collections.Counter()
    # family clustering pools
    fm_cluster = collections.Counter()   # (grp nabbr>0, n, nwrap_obs) for FM
    fw_cluster = collections.Counter()   # FW context
    ff_cluster = collections.Counter()
    fm_delta = collections.Counter()     # predicted-L minus nearest band edge (abbr-free)
    fm_examples = []
    fw_examples = []
    ff_examples = []
    for fname, kind, cells in rows:
        n = len(cells)
        widths = shipped_widths(cells)
        grp_abbr = sum(c["nabbr"] for c in cells)
        nwrap_obs = sum(1 for c in cells if c["wrap"])
        for i, c in enumerate(cells):
            b, pred_wrap = widths[i]
            L = 3 * b // 2
            if c["wrap"] and not pred_wrap:
                cnt["FF"] += 1
                ff_cluster[(grp_abbr > 0, n, c["nabbr"] > 0)] += 1
                if len(ff_examples) < 8:
                    ff_examples.append((fname, kind, n, i, c["f"], c["st"], b))
            elif not c["wrap"] and pred_wrap:
                cnt["FW"] += 1
                # does some OTHER cell in the row wrap (family-3 shape)?
                sib_wraps = any(cj["wrap"] for j, cj in enumerate(cells) if j != i)
                fw_cluster[(grp_abbr > 0, n, sib_wraps)] += 1
                if len(fw_examples) < 8:
                    fw_examples.append((fname, kind, n, i, c["f"], b,
                                        [(cj["f"], cj["st"][:12]) for cj in cells]))
            elif c["wrap"] and pred_wrap:
                if in_band(c["st"], L):
                    cnt["OK_fill"] += 1
                else:
                    cnt["FM"] += 1
                    fm_cluster[(grp_abbr > 0, n, nwrap_obs)] += 1
                    if grp_abbr == 0:
                        runs = band_runs(c["st"])
                        if runs:
                            # signed distance to nearest run edge
                            best = None
                            for a, bb in runs:
                                d = 0 if a <= L <= bb else (L - bb if L > bb else L - a)
                                if best is None or abs(d) < abs(best):
                                    best = d
                            fm_delta[max(-15, min(15, best))] += 1
                        else:
                            fm_delta["NONE"] += 1
                        if len(fm_examples) < 12 and n >= 2:
                            fm_examples.append((fname, kind, n, i, c["f"], c["st"], b, L,
                                                [(cj["f"], cj["tup"], cj["uni"], cj["wrap"]) for cj in cells]))
            else:
                cnt["OK_flat"] += 1
    print("counts:", dict(cnt))
    print("\nFM clusters (grp_has_abbr, ncells, n_obs_wrapping) -> count:")
    for k, v in fm_cluster.most_common(20):
        print("  ", k, v)
    print("\nFM abbr-free predicted-L minus nearest band edge:")
    for k in sorted(fm_delta, key=lambda x: (isinstance(x, str), x)):
        print("  ", k, fm_delta[k])
    print("\nFW clusters (grp_has_abbr, ncells, sibling_wraps) -> count:")
    for k, v in fw_cluster.most_common(12):
        print("  ", k, v)
    print("\nFF clusters (grp_has_abbr, ncells, cell_has_abbr) -> count:")
    for k, v in ff_cluster.most_common(12):
        print("  ", k, v)
    print("\nFM abbr-free examples (file kind n i flat st pred_b pred_L row[(f,tup,uni,wrap)]):")
    for e in fm_examples:
        print("  ", e)
    print("\nFW examples (file kind n i flat pred_b row[(f,st)]):")
    for e in fw_examples:
        print("  ", e)
    print("\nFF examples (file kind n i flat st pred_b):")
    for e in ff_examples:
        print("  ", e)

if __name__ == "__main__":
    main()
