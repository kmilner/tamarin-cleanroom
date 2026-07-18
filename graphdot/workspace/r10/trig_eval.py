#!/usr/bin/env python3
"""Trigger eval over bands3-format rows (9-field cells) for sqa-model variants.
Model: C_j = flat_j + at*dtop_j - kq*sqa_j ; budget_i = max(W + (4 if dtop_i>0 and n>=2 else 0) - sum_{j!=i} C_j, 20)
own eff_i = flat_i - (d*sqa_i if budget_i > 20 else 0); wrap iff eff > budget."""
import sys
def parse(path):
    groups=[]
    for line in open(path):
        p=line.rstrip("\n").split("\t")
        if len(p)!=6: continue
        cells=[]
        ok=True
        for c in p[5].split("|"):
            seg=c.split(":")
            if len(seg)!=9: ok=False; break
            flat=int(seg[0]); st=seg[1]
            dtop,drec,nq,sqa,nargs,nfunc,nabbr=map(int,seg[2:])
            cells.append(dict(f=flat, wrap=not st.startswith("F"), dtop=dtop, sqa=sqa))
        if ok and cells: groups.append(cells)
    return groups

def ev(groups, at, kq, d, bonus=True):
    fp=fn=tot=0
    for cs in groups:
        n=len(cs)
        if n<2: continue
        C=[c["f"]+at*c["dtop"]-kq*c["sqa"] for c in cs]
        S=sum(C)
        for c,Ci in zip(cs,C):
            b=87+(4 if (bonus and c["dtop"]>0) else 0)-(S-Ci)
            b=max(b,20)
            eff=c["f"]-(d*c["sqa"] if b>20 else 0)
            pw=eff>b; ow=c["wrap"]; tot+=1
            if pw and not ow: fp+=1
            elif not pw and ow: fn+=1
    return fp,fn,tot

groups=[]
for p in sys.argv[1:]: groups+=parse(p)
print(f"multi-cell rows from {len(groups)} groups")
for (at,kq,d,lbl) in [(2,0,2,"SHIPPED (dtop2,bonus,own-2)"),(2,0,0,"no-sqa-discount"),
                      (2,0,1,"own-1"),(2,4,0,"LOG C=flat-4"),(2,2,0,"occ-2"),
                      (0,0,0,"plain flat-sum"),(2,0,3,"own-3")]:
    fp,fn,tot=ev(groups,at,kq,d)
    print(f"  at={at} kq={kq} d={d:2d} [{lbl:28s}]: fp {fp:5d} fn {fn:5d} err {100*(fp+fn)/tot:.3f}% of {tot}")
