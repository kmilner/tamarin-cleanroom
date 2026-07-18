#!/usr/bin/env python3
# For each probe record, extract conclusion cells [Big, Sib]; for each cell,
# back out the effective line width W that the exact engine needs to reproduce
# the captured bytes. Report W vs (flat_Big, flat_Sib, group_total, sib_wraps).
import re, glob, subprocess, os
LAYOUT="/home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean/target/release/layout_at"
DOTS="/home/kamilner/tamarin-cleanroom/graphdot/workspace/r8/probe_dots"

def split_top(s, sep='|'):
    parts=[]; depth=0; cur=[]; i=0
    while i<len(s):
        c=s[i]
        if c=='\\' and i+1<len(s): cur.append(s[i:i+2]); i+=2; continue
        if c=='{': depth+=1; cur.append(c)
        elif c=='}': depth-=1; cur.append(c)
        elif c==sep and depth==0: parts.append(''.join(cur)); cur=[]
        else: cur.append(c)
        i+=1
    parts.append(''.join(cur)); return parts
def unescape(t):
    for a,b in [('\\<','<'),('\\>','>'),('\\{','{'),('\\}','}'),('\\|','|')]: t=t.replace(a,b)
    return t
def parse_record(label):
    if not (label.startswith('{') and label.endswith('}')): return None
    o=[]
    for g in split_top(label[1:-1]):
        g=g.strip()
        cs=[]
        for c in split_top(g[1:-1]):
            m=re.match(r'<n\d+> (.*)$', c, re.S)
            if not m: return None
            cs.append(m.group(1))
        o.append(cs)
    return o
def dewrap(cell):
    t=cell.replace('&nbsp;','')
    t=t.replace(',\\l', ', '); t=t.replace('\\l)', ' )'); t=t.replace('\\l','')
    return unescape(t)
def flat_width_cell(cell):
    lost=len(re.findall(r',\\l',cell))+len(re.findall(r'\\l\)',cell))
    return len(unescape(cell.replace('\\l','').replace('&nbsp;','')))+lost

def find_width(flat, actual, lo=12, hi=90):
    out=subprocess.run([LAYOUT, flat, str(lo), str(hi)], capture_output=True, text=True).stdout
    hits=[]
    for line in out.splitlines():
        w,r=line.split('\t',1)
        if r==actual: hits.append(int(w))
    return hits

rows=[]
for f in sorted(glob.glob(DOTS+"/l_R_*.dot")):
    name=os.path.basename(f)[2:-4]  # R_K_P
    label=None
    for line in open(f):
        if 'shape="record"' in line:
            m=re.search(r'label="(.*)",fillcolor', line); label=m.group(1); break
    if not label: continue
    groups=parse_record(label)
    if not groups: continue
    concl=groups[-1]  # conclusion group
    # cells: <nK> body ; but parse_record stripped the port already
    bodies=concl
    flats=[flat_width_cell(b) for b in bodies]
    T=sum(flats)
    for k,b in enumerate(bodies):
        wraps = '\\l' in b
        tag = 'Big' if b.strip().startswith('Big') else ('Sib' if b.strip().startswith('Sib') else f'c{k}')
        if not wraps: continue
        flat=dewrap(b)
        ws=find_width(flat, b)
        sib_flat=flats[1-k] if len(flats)==2 else None
        rows.append((name, tag, flats[k], sib_flat, T, ws))

print(f"{'name':12} {'cell':4} {'flat':>4} {'sibf':>4} {'T':>4}  Wrange   87-sib  87-T+flat")
for name,tag,flat,sibf,T,ws in rows:
    wr = f"{min(ws)}-{max(ws)}" if ws else "NONE"
    b1 = 87-sibf if sibf is not None else None
    b2 = 87-(T-flat)
    print(f"{name:12} {tag:4} {flat:4d} {str(sibf):>4} {T:4d}  {wr:8} {str(b1):>6} {b2:>6}")
