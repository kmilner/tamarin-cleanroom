#!/usr/bin/env python3
"""Per-cell eval of trigger+fill variants over b3 TSVs (probe batteries)."""
import sys
def parse(path):
    rows=[]
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
            if st.startswith("F"):
                lo=None if st=="FNONE" else int(st[1:])
                cells.append(dict(f=flat,wrap=False,lo=lo,dtop=dtop,sqa=sqa))
            elif st=="NONE":
                cells.append(dict(f=flat,wrap=True,runs=None,dtop=dtop,sqa=sqa))
            else:
                runs=[tuple(map(int,r.split("-"))) for r in st.split(",")]
                cells.append(dict(f=flat,wrap=True,runs=runs,dtop=dtop,sqa=sqa))
        if ok and cells: rows.append((p[0],cells))
    return rows

def model(cells, own_d):
    """Return per-cell (pred_wrap, L) mirroring generate::group_widths."""
    n=len(cells)
    out=[]
    C=[c["f"]+c["dtop"] for c in cells]
    S=sum(C)
    for i,c in enumerate(cells):
        f=c["f"]
        if n==1:
            out.append((f>87, 3*87//2)); continue
        bonus=4 if c["dtop"]>0 else 0
        bt=max(87+bonus-(S-C[i]),20)
        eff=f-(own_d if (c["sqa"] and bt>20) else 0)
        if eff<=bt:
            b=max(bt,f); out.append((False, 3*b//2)); continue
        t=float(f)
        for j in range(n):
            if j!=i: t+=(5/6 if cells[j]["sqa"] else 1.0)*cells[j]["f"]
        b=int(87.0*f/t+0.5)
        b=max(b,20); b=min(b,max(f-1,20))
        out.append((True, 3*b//2))
    return out

def inband(c,L):
    if not c["wrap"]: return c["lo"] is not None and L>=c["lo"]
    if c["runs"] is None: return False
    return any(a<=L<=b for a,b in c["runs"])

for own_d in (2,0):
    print(f"=== own_d={own_d}")
    trig_err=fill_miss=trig_tot=fill_tot=0
    for path in sys.argv[1:]:
        for fname,cells in parse(path):
            preds=model(cells,own_d)
            for i,(c,(pw,L)) in enumerate(zip(cells,preds)):
                trig_tot+=1
                if pw!=c["wrap"]:
                    trig_err+=1
                    print(f"  TRIG {fname} cell{i} flat={c['f']} obs={'W' if c['wrap'] else 'F'} pred={'W' if pw else 'F'}")
                elif c["wrap"]:
                    fill_tot+=1
                    if not inband(c,L):
                        fill_miss+=1
                        print(f"  FILL {fname} cell{i} flat={c['f']} L={L} runs={c['runs']}")
    print(f"  trigger {trig_tot-trig_err}/{trig_tot}  fill-band {fill_tot-fill_miss}/{fill_tot}")
