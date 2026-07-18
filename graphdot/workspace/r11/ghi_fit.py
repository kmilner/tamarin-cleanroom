#!/usr/bin/env python3
"""Score candidate allocation models on the GHI probe bands and print the raw
per-row inventory (flats + observed ribbon bands + model predictions)."""
import sys, math
from eval import parse, band_runs, in_band
from seqfit import C_of, N_of, prop, clamp, MODELS

def l2r(L):
    v = L / 1.5
    fl = math.floor(v)
    if abs(v - fl - 0.5) < 1e-9:
        f = int(fl)
        return f if f % 2 == 0 else f + 1
    return round(v)

def rr(st):
    if st.startswith("F"):
        return st
    rs = band_runs(st)
    if not rs:
        return "NONE"
    return ",".join(f"{l2r(a)}..{l2r(b)}" for a, b in rs)

def main():
    rows = parse(sys.argv[1] if len(sys.argv) > 1 else "bandsGHI.tsv")
    scores = {name: [0, 0] for name in MODELS}
    for fname, kind, cells in sorted(rows):
        wraps = [c["wrap"] for c in cells]
        desc = " ".join(f"[{c['f']}{'+t' + str(c['tup']) if c['tup'] else ''} "
                        f"{'W' if c['wrap'] else 'flat'} {rr(c['st'])}]" for c in cells)
        preds = []
        for name, fn in MODELS.items():
            bs = fn(cells, wraps)
            oks = [in_band(c["st"], 3 * b // 2) for c, b in zip(cells, bs) if b is not None]
            scores[name][0] += sum(oks)
            scores[name][1] += len(oks)
            miss = [(c["f"], b) for c, b in zip(cells, bs)
                    if b is not None and not in_band(c["st"], 3 * b // 2)]
            preds.append(f"{name}={'OK' if not miss else miss}")
        print(fname.replace(".dot", ""), desc, " | ".join(preds))
    print()
    for name, (h, t) in scores.items():
        print(f"{name}: {h}/{t}")

if __name__ == "__main__":
    main()
