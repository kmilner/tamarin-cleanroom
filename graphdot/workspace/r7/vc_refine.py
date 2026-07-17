#!/usr/bin/env python3
"""Test conservative occ-relief refinements of the flat-sum trigger against the
corpus (using ACTUAL occ from the rendering as the best case).
M0: flatsum.
Ma: flatsum, but OVERRIDE to FIT if flat_i <= 87 - sum(occ_j) AND i is strictly
    smaller than every sibling that wraps (the disparate small-cell case).
Mb: like Ma but also require i to be the unique global min-flat cell in the group.
"""
import re, glob
CORP="/home/kamilner/tamarin-cleanroom/graphdot/oracle/dot_corpus"
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
def phys(cell):
    if '\\l' not in cell: return [len(unescape(cell))]
    segs=cell.split('\\l');
    if segs and segs[-1]=='': segs=segs[:-1]
    o=[]
    for s in segs:
        m=re.match(r'((?:&nbsp;)*)(.*)$', s, re.S); o.append(len(m.group(1))//6+len(unescape(m.group(2))))
    return o
def flat_width(cell):
    lost=len(re.findall(r',\\l',cell))+len(re.findall(r'\\l\)',cell))
    return len(unescape(cell.replace('\\l','').replace('&nbsp;',''))) + lost
def occ(cell): return max(phys(cell))
def parse_record(label):
    if not (label.startswith('{') and label.endswith('}')): return None
    o=[]
    for g in split_top(label[1:-1]):
        g=g.strip()
        if not (g.startswith('{') and g.endswith('}')): return None
        cs=[]
        for c in split_top(g[1:-1]):
            m=re.match(r'<n\d+> (.*)$', c, re.S)
            if not m: return None
            cs.append(m.group(1))
        o.append(cs)
    return o
def is_info(cells): return len(cells)==1 and re.match(r'#\S+ : ', cells[0])

tot=0; m0=0; ma=0; mb=0
for f in sorted(glob.glob(CORP+"/*.dot")):
    for line in open(f):
        if 'shape="record"' not in line: continue
        m=re.search(r'label="(.*)",fillcolor', line)
        if not m: continue
        groups=parse_record(m.group(1))
        if groups is None: continue
        for cells in groups:
            if is_info(cells): continue
            flats=[flat_width(c) for c in cells]; occs=[occ(c) for c in cells]
            wraps=['\\l' in c for c in cells]
            T=sum(flats); To=sum(occs); minf=min(flats)
            for k,c in enumerate(cells):
                actual=wraps[k]; fw=flats[k]
                b0=max(87-(T-fw),20); p0=fw>b0
                # override candidate
                sib_occ_sum=To-occs[k]
                wrapping_sibs=[flats[j] for j in range(len(cells)) if j!=k and wraps[j]]
                fits_by_occ = fw <= max(87-sib_occ_sum,20)
                smaller_than_wrapsibs = wrapping_sibs and all(fw < s for s in wrapping_sibs)
                pa = p0 and not (fits_by_occ and smaller_than_wrapsibs)
                pb = p0 and not (fits_by_occ and smaller_than_wrapsibs and fw==minf and flats.count(minf)==1)
                tot+=1
                m0 += (p0!=actual); ma += (pa!=actual); mb += (pb!=actual)
print(f"cells {tot}")
print(f"M0 flatsum         : {m0} ({100*m0/tot:.4f}%)")
print(f"Ma occ-relief guard: {ma} ({100*ma/tot:.4f}%)")
print(f"Mb +unique-min     : {mb} ({100*mb/tot:.4f}%)")
