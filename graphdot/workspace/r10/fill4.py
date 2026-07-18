#!/usr/bin/env python3
"""Fill-model eval over b4 TSVs (11-field). Trigger fixed at the round-10 law:
C = flat+ctup, bonus = bmax if bmax<=6 else 4, floor 20, lone 87."""
import sys, math
from trig4 import parse
def inband(c,L):
    st=c["st"]
    if st.startswith("F"):
        return st!="FNONE" and L>=int(st[1:])
    if st=="NONE": return False
    return any(a<=L<=b for a,b in (tuple(map(int,r.split("-"))) for r in st.split(",")))
def bonus(c): return c["bmax"] if c["bmax"]<=6 else 4
def run(rowsets, num, rnd, wsqa):
    te=fh=ft=0
    for rows in rowsets:
        for fname,cells in rows:
            n=len(cells)
            C=[c["f"]+c["ctup"] for c in cells]; S=sum(C)
            for i,c in enumerate(cells):
                b_t=87 if n==1 else max(87+bonus(c)-(S-C[i]),20)
                pw=c["f"]>b_t
                if pw!=c["wrap"]: te+=1; continue
                if not c["wrap"]: continue
                ft+=1
                if n==1: b=87
                else:
                    top=float(num(c))
                    t=top
                    for j,cj in enumerate(cells):
                        if j!=i:
                            w=wsqa if (cj.get("sqa") or cj.get("nq",0)==1 and False) else 1.0
                            w=wsqa if cj.get("sqa") else 1.0
                            t+=w*cj["f"]
                    x=87.0*top/t
                    b=int(math.floor(x)) if rnd=="f" else (int(x+0.5) if rnd=="h" else round(x))
                    b=max(20,min(b,max(c["f"]-1,20)))
                if inband(c,3*b//2): fh+=1
    return te,fh,ft
# need sqa flag: re-parse keeping sqa
import trig4
def parse2(path):
    rows=[]
    for line in open(path):
        p=line.rstrip("\n").split("\t")
        if len(p)!=6: continue
        cells=[]; ok=True
        for cc in p[5].split("|"):
            seg=cc.split(":")
            if len(seg)!=11: ok=False; break
            flat=int(seg[0]); st=seg[1]
            dtop,drec,nq,sqa,nargs,nfunc,nabbr,ctup,bmax=map(int,seg[2:])
            cells.append(dict(f=flat,wrap=not st.startswith("F"),dtop=dtop,sqa=sqa,
                              ctup=ctup,bmax=bmax,st=st,nabbr=nabbr,nfunc=nfunc))
        if ok and cells: rows.append((p[0],cells))
    return rows
probe_paths=["b4_probeA_dots.tsv","b4_probeB_dots.tsv","b4_probeC_dots.tsv",
             "b4_probeE_dots.tsv","b4_probeF_dots.tsv","b4_probe2_dots.tsv",
             "b4_probe3_dots.tsv","b4_probe4_dots.tsv","b4_probe_dots.tsv"]
probes=[parse2(p) for p in probe_paths]
corpus=[parse2("bands4.tsv")]
variants=[
 ("unified C+nfunc, half-up",  lambda c: c["f"]+c["ctup"]+c["nfunc"], "h", 1.0),
 ("unified C+nfunc, floor",    lambda c: c["f"]+c["ctup"]+c["nfunc"], "f", 1.0),
 ("unified C, half-up",        lambda c: c["f"]+c["ctup"], "h", 1.0),
 ("unified C+2nfunc, half-up", lambda c: c["f"]+c["ctup"]+2*c["nfunc"], "h", 1.0),
 ("flat numerator, half-up",   lambda c: c["f"], "h", 1.0),
 ("v7ish flat+5/6sqa, half-up",lambda c: c["f"], "h", 5/6),
]
for name,num,rnd,ws in variants:
    te,fh,ft=run(probes,num,rnd,ws)
    tec,fhc,ftc=run(corpus,num,rnd,ws)
    print(f"{name:28s}: probes trigmiss {te:2d} fill {fh}/{ft}={100*fh/max(ft,1):.1f}% | corpus fill {fhc}/{ftc}={100*fhc/ftc:.2f}%")
