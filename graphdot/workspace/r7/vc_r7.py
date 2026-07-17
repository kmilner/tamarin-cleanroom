#!/usr/bin/env python3
"""Corpus validation of the record-cell wrap TRIGGER.
Model 0 (Session-6 flat-sum):  wrap_i  <=>  flat_i > max(87 - sum flat_j, 20)
Model 1 (occ):                 wrap_i  <=>  flat_i > max(87 - sum occ_j, 20)
where occ_j = the actual occupied (max physical-line) width of sibling j in the
rendered DOT (= flat_j when j is unwrapped).  occ is read from the corpus itself,
so Model 1 tests whether the actual layout is a consistent fixpoint of the rule.
"""
import re, glob, sys

CORP = "/home/kamilner/tamarin-cleanroom/graphdot/oracle/dot_corpus"

def split_top(s, sep='|'):
    parts=[]; depth=0; cur=[]; i=0
    while i < len(s):
        c=s[i]
        if c=='\\' and i+1<len(s):
            cur.append(s[i:i+2]); i+=2; continue
        if c=='{': depth+=1; cur.append(c)
        elif c=='}': depth-=1; cur.append(c)
        elif c==sep and depth==0:
            parts.append(''.join(cur)); cur=[]
        else: cur.append(c)
        i+=1
    parts.append(''.join(cur))
    return parts

def unescape(t):
    for a,b in [('\\<','<'),('\\>','>'),('\\{','{'),('\\}','}'),('\\|','|')]:
        t=t.replace(a,b)
    return t

def phys_lines(cell):
    """Return list of physical-line widths (indent cols + text cols)."""
    if '\\l' not in cell:
        return [len(unescape(cell))]
    segs = cell.split('\\l')
    if segs and segs[-1]=='': segs=segs[:-1]
    out=[]
    for s in segs:
        m=re.match(r'((?:&nbsp;)*)(.*)$', s, re.S)
        ind=len(m.group(1))//len('&nbsp;')
        out.append(ind + len(unescape(m.group(2))))
    return out

def flat_width(cell):
    lost = len(re.findall(r',\\l', cell)) + len(re.findall(r'\\l\)', cell))
    t = cell.replace('\\l','').replace('&nbsp;','')
    return len(unescape(t)) + lost

def occ_width(cell):
    return max(phys_lines(cell))

def parse_record(label):
    if not (label.startswith('{') and label.endswith('}')): return None
    groups=split_top(label[1:-1])
    parsed=[]
    for g in groups:
        g=g.strip()
        if not (g.startswith('{') and g.endswith('}')): return None
        cells=split_top(g[1:-1])
        texts=[]
        for c in cells:
            m=re.match(r'<n\d+> (.*)$', c, re.S)
            if not m: return None
            texts.append(m.group(1))
        parsed.append(texts)
    return parsed

def is_info(cells):
    return len(cells)==1 and re.match(r'#\S+ : ', cells[0])

tot=0; m0=0; m1=0
ex0=[]; ex1=[]
distr0={}
files=sorted(glob.glob(CORP+"/*.dot"))
for f in files:
    for line in open(f):
        if 'shape="record"' not in line: continue
        m=re.search(r'label="(.*)",fillcolor', line)
        if not m: continue
        groups=parse_record(m.group(1))
        if groups is None: continue
        for gi,cells in enumerate(groups):
            if is_info(cells): continue  # info handled separately; trigger study on prem/concl
            flats=[flat_width(c) for c in cells]
            occs=[occ_width(c) for c in cells]
            T=sum(flats)
            Tocc=sum(occs)
            for k,c in enumerate(cells):
                actual = '\\l' in c
                fw=flats[k]
                b0=max(87-(T-fw),20)
                b1=max(87-(Tocc-occs[k]),20)
                p0 = fw>b0
                p1 = fw>b1
                tot+=1
                if p0!=actual:
                    m0+=1
                    distr0[T]=distr0.get(T,0)+1
                    if len(ex0)<8: ex0.append((f.split('/')[-1],T,fw,occs,p0,actual,c[:70]))
                if p1!=actual:
                    m1+=1
                    if len(ex1)<15: ex1.append((f.split('/')[-1],T,Tocc,fw,flats,occs,p1,actual,c[:70]))

print(f"prem/concl cells: {tot}")
print(f"Model0 flat-sum  mismatches: {m0} ({100*m0/tot:.4f}%)  match {100*(1-m0/tot):.4f}%")
print(f"Model1 occ       mismatches: {m1} ({100*m1/tot:.4f}%)  match {100*(1-m1/tot):.4f}%")
print("\nModel1 residual examples:")
for e in ex1:
    print("  ", e[:8], repr(e[8]))
