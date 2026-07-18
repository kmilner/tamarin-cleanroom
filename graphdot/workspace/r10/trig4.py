#!/usr/bin/env python3
"""Trigger-variant eval over 11-field b4 TSVs."""
import sys
def parse(path):
    rows=[]
    for line in open(path):
        p=line.rstrip("\n").split("\t")
        if len(p)!=6: continue
        cells=[]; ok=True
        for c in p[5].split("|"):
            seg=c.split(":")
            if len(seg)!=11: ok=False; break
            flat=int(seg[0]); st=seg[1]
            dtop,drec,nq,sqa,nargs,nfunc,nabbr,ctup,bmax=map(int,seg[2:])
            cells.append(dict(f=flat,wrap=not st.startswith("F"),dtop=dtop,
                              ctup=ctup,bmax=bmax,st=st,nabbr=nabbr))
        if ok and cells: rows.append((p[0],cells))
    return rows

def ev(rowsets, model, verbose=False):
    fp=fn=tot=0; misses=[]
    for rows in rowsets:
        for fname,cells in rows:
            n=len(cells)
            if n<2: continue
            if model=="old":
                C=[c["f"]+c["dtop"] for c in cells]
                B=[4 if c["dtop"]>0 else 0 for c in cells]
            elif model=="new":
                C=[c["f"]+c["ctup"] for c in cells]
                B=[c["bmax"] for c in cells]
            elif model=="newcap":
                C=[c["f"]+c["ctup"] for c in cells]
                B=[min(c["bmax"],11) for c in cells]
            elif model=="flat":
                C=[c["f"] for c in cells]
                B=[0]*n
            S=sum(C)
            for c,Ci,Bi in zip(cells,C,B):
                b=max(87+Bi-(S-Ci),20)
                pw=c["f"]>b; ow=c["wrap"]; tot+=1
                if pw!=ow:
                    if pw: fp+=1
                    else: fn+=1
                    misses.append((fname,c["f"],c["st"][:12],"predW" if pw else "predF"))
    return fp,fn,tot,misses

probe_paths=["b4_probeA_dots.tsv","b4_probeB_dots.tsv","b4_probeC_dots.tsv",
             "b4_probeE_dots.tsv","b4_probeF_dots.tsv","b4_probe2_dots.tsv",
             "b4_probe3_dots.tsv","b4_probe4_dots.tsv","b4_probe_dots.tsv"]
probes=[parse(p) for p in probe_paths]
corpus=[parse("bands4.tsv")]
for m in ("old","new","newcap","flat"):
    fp,fn,tot,mis=ev(probes,m)
    fpc,fnc,totc,_=ev(corpus,m)
    print(f"{m:7s}: probes fp {fp:3d} fn {fn:3d} of {tot} ({tot-fp-fn} ok)  | corpus fp {fpc} fn {fnc} err {100*(fpc+fnc)/totc:.3f}%")
    if m=="new":
        for x in mis[:20]: print("   ",x)
