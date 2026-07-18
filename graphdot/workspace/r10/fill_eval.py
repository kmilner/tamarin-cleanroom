#!/usr/bin/env python3
"""Fill-share variant eval over probe b3 TSVs. Trigger fixed: C=flat+dtop,
bonus+4 (dtop>0, n>=2), floor 20, own eff = flat (audit outcome, no sqa disc)."""
import sys, math
from probe_eval import parse, inband

def fill_b(cells, i, variant):
    f = cells[i]["f"]
    t = float(f)
    for j,c in enumerate(cells):
        if j==i: continue
        w = 1.0
        if c["sqa"]:
            if variant in ("ship","v3"): w = 5/6
            elif variant in ("v4","v5"): w = 5/6 if cells[i]["dtop"]>0 else 1.0
        if variant in ("v3","v4","v5") and w != 1.0:
            t += math.floor(w*c["f"])
        else:
            t += w*c["f"]
    x = 87.0*f/t
    if variant in ("ship","v2"): b = int(x+0.5)
    elif variant=="v5" and cells[i]["dtop"]==0: b = int(x+0.5)
    else: b = int(math.floor(x))
    return max(20, min(b, max(f-1,20)))

def run(paths, variant):
    trig_err=fill_hit=fill_tot=0
    misses=[]
    for path in paths:
        for fname,cells in parse(path):
            n=len(cells)
            C=[c["f"]+c["dtop"] for c in cells]
            S=sum(C)
            for i,c in enumerate(cells):
                if n==1:
                    pw = c["f"]>87; b=87
                else:
                    bt=max(87+(4 if c["dtop"]>0 else 0)-(S-C[i]),20)
                    pw = c["f"]>bt
                if pw!=c["wrap"]:
                    trig_err+=1; continue
                if not c["wrap"]: continue
                fill_tot+=1
                b = fill_b(cells,i,variant) if n>1 else 87
                L = 3*b//2
                if inband(c,L): fill_hit+=1
                else: misses.append((fname,i,c["f"],b,L,c["runs"]))
    return trig_err, fill_hit, fill_tot, misses

paths = sys.argv[1:]
for v in ("ship","v1","v2","v3","v4","v5"):
    te,fh,ft,misses = run(paths,v)
    print(f"{v:5s}: trig_err {te}  fill {fh}/{ft} = {100*fh/max(ft,1):.1f}%")
    if v in ("v4",):
        for m in misses[:14]: print("   miss:",m)

def fill_b7(cells, i):
    import math
    f = cells[i]["f"]
    t = float(f)
    for j,c in enumerate(cells):
        if j==i: continue
        w = 5/6 if (c["sqa"] and cells[i]["dtop"]>0) else 1.0
        t += w*c["f"]
    b = int(87.0*f/t+0.5)
    return max(20, min(b, max(f-1,20)))
