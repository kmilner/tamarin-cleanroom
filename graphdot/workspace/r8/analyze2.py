#!/usr/bin/env python3
# Faster: cache the width->render sweep per unique flat string, then for each
# wrapping conclusion cell find the effective width band.
import re, glob, subprocess, os, sys
LAYOUT="/home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean/target/release/layout_at"
DOTS="/home/kamilner/tamarin-cleanroom/graphdot/workspace/r8/probe_dots"
LO,HI=18,87

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
        g=g.strip(); cs=[]
        for c in split_top(g[1:-1]):
            m=re.match(r'<n\d+> (.*)$', c, re.S)
            if not m: return None
            cs.append(m.group(1))
        o.append(cs)
    return o
def dewrap(cell):
    t=cell.replace('&nbsp;','').replace(',\\l', ', ').replace('\\l)', ' )').replace('\\l','')
    return unescape(t)
def flat_width_cell(cell):
    lost=len(re.findall(r',\\l',cell))+len(re.findall(r'\\l\)',cell))
    return len(unescape(cell.replace('\\l','').replace('&nbsp;','')))+lost

cache={}
def sweep(flat):
    if flat in cache: return cache[flat]
    out=subprocess.run([LAYOUT, flat, str(LO), str(HI)], capture_output=True, text=True).stdout
    d={}
    for line in out.splitlines():
        w,r=line.split('\t',1); d[r]=d.get(r,[]); d[r].append(int(w))
    cache[flat]=d; return d

rows=[]
for f in sorted(glob.glob(DOTS+"/l_R_*.dot")):
    name=os.path.basename(f)[2:-4]
    label=None
    for line in open(f):
        if 'shape="record"' in line:
            m=re.search(r'label="(.*)",fillcolor', line); label=m.group(1); break
    if not label: continue
    groups=parse_record(label)
    if not groups: continue
    concl=groups[-1]
    flats=[flat_width_cell(b) for b in concl]
    T=sum(flats)
    for k,b in enumerate(concl):
        if '\\l' not in b: continue
        tag='Big' if b.strip().startswith('Big') else ('Sib' if b.strip().startswith('Sib') else f'c{k}')
        flat=dewrap(b)
        d=sweep(flat)
        ws=d.get(b, [])
        sib_flat=flats[1-k] if len(flats)==2 else None
        rows.append((name,tag,flats[k],sib_flat,T,ws))

print(f"{'name':10} {'cell':4} {'flat':>4} {'sibf':>4} {'T':>4}  {'Wrange':>8} {'87-sib':>7} {'87-(T-flat)':>11}")
for name,tag,flat,sibf,T,ws in rows:
    wr=f"{min(ws)}-{max(ws)}" if ws else "NONE"
    b1=87-sibf if sibf is not None else -1
    b2=87-(T-flat)
    print(f"{name:10} {tag:4} {flat:4d} {str(sibf):>4} {T:4d}  {wr:>8} {b1:>7} {b2:>11}")
