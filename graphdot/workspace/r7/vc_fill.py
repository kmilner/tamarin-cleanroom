#!/usr/bin/env python3
"""Measure how well a per-cell FILL budget reproduces the actual wrapped layout of
prem/concl cells across the corpus.  For each wrapping cell we reconstruct its
flat text, re-run the crate's fill at a candidate budget, and compare the result
to the actual cell bytes (both full and line-0-only)."""
import re, glob, sys
sys.path.insert(0,'/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from fillsim import layout_cell, cell_occ
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

def dewrap(cell):
    t=cell.replace('&nbsp;','')
    t=t.replace(',\\l', ', ')   # fact-arg / info break dropped the space
    t=t.replace('\\l)', ' )')   # fact ')' peel: flat had ' )'
    t=t.replace('\\l','')       # tuple '>' peel, tuple-elem break ', ', joins
    return unescape(t)

def flat_width_cell(cell):
    lost=len(re.findall(r',\\l',cell))+len(re.findall(r'\\l\)',cell))
    return len(unescape(cell.replace('\\l','').replace('&nbsp;',''))) + lost

def render_lines(flat, budget):
    lines=layout_cell(flat,budget)
    if len(lines)==1 and lines[0][0]==0: return flat  # single line (unescaped)
    return "|".join(f"{ind}:{txt}" for ind,txt in lines)

def actual_lines(cell):
    if '\\l' not in cell: return unescape(cell)
    segs=cell.split('\\l')
    if segs and segs[-1]=='': segs=segs[:-1]
    out=[]
    for s in segs:
        m=re.match(r'((?:&nbsp;)*)(.*)$', s, re.S)
        out.append(f"{len(m.group(1))//6}:{unescape(m.group(2))}")
    return "|".join(out)

def cell_budget(flats,i):
    others=sum(f for j,f in enumerate(flats) if j!=i)
    return max(87-others,20)

def occ_budgets(flat_cells, flats):
    n=len(flats)
    order=sorted(range(n), key=lambda i: flats[i])
    occ=list(flats); fill=[0]*n
    for i in order:
        others=sum(occ[j] for j in range(n) if j!=i)
        b=max(87-others,20); fill[i]=b
        occ[i]=cell_occ(flat_cells[i], b)
    return fill

def minbud_budgets(flats):
    # each sibling occupies min(flat, its flat-sum budget): a wrapping sibling
    # fills its allocated budget, a fitting one its flat.
    n=len(flats)
    contrib=[min(flats[j], cell_budget(flats,j)) for j in range(n)]
    return [max(87-(sum(contrib)-contrib[i]),20) for i in range(n)]

def greedy_budgets(flats):
    # smallest-flat-first; a processed sibling occupies min(flat, its assigned
    # budget), an un-processed (wider) one its full flat.
    n=len(flats)
    order=sorted(range(n), key=lambda i: flats[i])
    contrib=list(flats); budget=[0]*n
    for i in order:
        others=sum(contrib[j] for j in range(n) if j!=i)
        b=max(87-others,20); budget[i]=b
        contrib[i]=min(flats[i], b)
    return budget

# candidate budgets: flatsum, occ-greedy
tot=0
match_flat=0; match_occ=0; match_min=0
wide_report=[]
mism_examples=[]
for f in sorted(glob.glob(CORP+"/*.dot")):
    for line in open(f):
        if 'shape="record"' not in line: continue
        m=re.search(r'label="(.*)",fillcolor', line)
        if not m: continue
        groups=parse_record(m.group(1))
        if groups is None: continue
        for cells in groups:
            if is_info(cells): continue
            flats=[flat_width_cell(c) for c in cells]
            flat_cells=[dewrap(c) for c in cells]
            occb=occ_budgets(flat_cells, flats)
            grb=greedy_budgets(flats)
            for k,c in enumerate(cells):
                if '\\l' not in c: continue  # only wrapping cells (fill matters)
                tot+=1
                act=actual_lines(c)
                bf=cell_budget(flats,k)
                def eff(b): return min(b if flats[k]>bf else bf, flats[k]-1)
                rf=render_lines(flat_cells[k], min(bf, flats[k]-1))
                ro=render_lines(flat_cells[k], eff(occb[k]))
                rg=render_lines(flat_cells[k], eff(grb[k]))
                mf=(rf==act); mo=(ro==act); mm=(rg==act)
                match_flat+=mf; match_occ+=mo; match_min+=mm
print(f"wrapping prem/concl cells: {tot}")
print(f"flatsum            budget match: {match_flat} ({100*match_flat/tot:.3f}%)")
print(f"occ-greedy         budget match: {match_occ} ({100*match_occ/tot:.3f}%)")
print(f"greedy min(flat,b) budget match: {match_min} ({100*match_min/tot:.3f}%)")
