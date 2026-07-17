#!/usr/bin/env python3
"""Characterise the flat-sum TRIGGER residual: split by direction, T, group size,
and whether a sibling wraps.  Goal: find the size-dependent refinement."""
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
        elif c==sep and depth==0: parts.append(''.join(cur)); cur=[]
        else: cur.append(c)
        i+=1
    parts.append(''.join(cur)); return parts

def unescape(t):
    for a,b in [('\\<','<'),('\\>','>'),('\\{','{'),('\\}','}'),('\\|','|')]: t=t.replace(a,b)
    return t
def phys_lines(cell):
    if '\\l' not in cell: return [len(unescape(cell))]
    segs=cell.split('\\l');
    if segs and segs[-1]=='': segs=segs[:-1]
    out=[]
    for s in segs:
        m=re.match(r'((?:&nbsp;)*)(.*)$', s, re.S); out.append(len(m.group(1))//6 + len(unescape(m.group(2))))
    return out
def flat_width(cell):
    lost=len(re.findall(r',\\l',cell))+len(re.findall(r'\\l\)',cell))
    return len(unescape(cell.replace('\\l','').replace('&nbsp;',''))) + lost
def occ_width(cell): return max(phys_lines(cell))
def parse_record(label):
    if not (label.startswith('{') and label.endswith('}')): return None
    parsed=[]
    for g in split_top(label[1:-1]):
        g=g.strip()
        if not (g.startswith('{') and g.endswith('}')): return None
        texts=[]
        for c in split_top(g[1:-1]):
            m=re.match(r'<n\d+> (.*)$', c, re.S)
            if not m: return None
            texts.append(m.group(1))
        parsed.append(texts)
    return parsed
def is_info(cells): return len(cells)==1 and re.match(r'#\S+ : ', cells[0])

# residual buckets
false_pos=0  # pred wrap, actual fit
false_neg=0  # pred fit, actual wrap
fp_sibwrap=0  # false_pos where some sibling wraps
fp_T={}; fn_T={}
fp_ex=[]; fn_ex=[]
tot=0
for f in sorted(glob.glob(CORP+"/*.dot")):
    for line in open(f):
        if 'shape="record"' not in line: continue
        m=re.search(r'label="(.*)",fillcolor', line)
        if not m: continue
        groups=parse_record(m.group(1))
        if groups is None: continue
        for cells in groups:
            if is_info(cells): continue
            flats=[flat_width(c) for c in cells]
            T=sum(flats)
            anywrap=any('\\l' in c for c in cells)
            for k,c in enumerate(cells):
                actual='\\l' in c; fw=flats[k]
                b0=max(87-(T-fw),20); p0=fw>b0
                tot+=1
                if p0 and not actual:
                    false_pos+=1; fp_T[T]=fp_T.get(T,0)+1
                    sw=any(('\\l' in cc) for j,cc in enumerate(cells) if j!=k)
                    if sw: fp_sibwrap+=1
                    if len(fp_ex)<12: fp_ex.append((f.split('/')[-1],T,fw,[flat_width(x) for x in cells],[('W' if '\\l' in x else 'f') for x in cells],c[:65]))
                elif (not p0) and actual:
                    false_neg+=1; fn_T[T]=fn_T.get(T,0)+1
                    if len(fn_ex)<12: fn_ex.append((f.split('/')[-1],T,fw,[flat_width(x) for x in cells],c[:65]))

print(f"total prem/concl cells: {tot}")
print(f"false_pos (pred WRAP, actual FIT): {false_pos}   of which a sibling wraps: {fp_sibwrap}")
print(f"false_neg (pred FIT, actual WRAP): {false_neg}")
print(f"\nfalse_pos T-distribution: {dict(sorted(fp_T.items()))}")
print(f"\nfalse_neg T-distribution: {dict(sorted(fn_T.items()))}")
print("\nfalse_pos examples (T, flat, sib-flats, wrapflags, cell):")
for e in fp_ex: print("  ",e)
print("\nfalse_neg examples:")
for e in fn_ex: print("  ",e)
