#!/usr/bin/env python3
"""Test the UNABBREVIATION hypothesis for the flat-sum false_neg residual:
does a cell wrap (despite displayed group-total <= 87) because its UNABBREVIATED
width exceeds its budget?  Expand each graph's legend (NAME = EXPANSION, nested)
and recompute cell widths on the fully-expanded text."""
import re, glob, sys
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
def flat_width(cell):
    lost=len(re.findall(r',\\l',cell))+len(re.findall(r'\\l\)',cell))
    return len(unescape(cell.replace('\\l','').replace('&nbsp;',''))) + lost
def flat_text(cell):
    lost=len(re.findall(r',\\l',cell))+len(re.findall(r'\\l\)',cell))
    return unescape(cell.replace('\\l','').replace('&nbsp;','')), lost
def parse_record(label):
    if not (label.startswith('{') and label.endswith('}')): return None
    out=[]
    for g in split_top(label[1:-1]):
        g=g.strip()
        if not (g.startswith('{') and g.endswith('}')): return None
        cs=[]
        for c in split_top(g[1:-1]):
            m=re.match(r'<n\d+> (.*)$', c, re.S)
            if not m: return None
            cs.append(m.group(1))
        out.append(cs)
    return out
def is_info(cells): return len(cells)==1 and re.match(r'#\S+ : ', cells[0])

def legend_map(dot):
    # rows: <FONT COLOR="#000000">NAME</FONT>...<TD ...>=</TD>...<TD ...>EXPANSION</TD>
    m={}
    for row in re.finditer(r'<FONT COLOR="#000000">([^<]+)</FONT></TD>\s*<TD[^>]*>=</TD>\s*<TD[^>]*>(.*?)</TD>', dot):
        name=row.group(1)
        exp=row.group(2)
        # HTML unescape
        exp=exp.replace('&lt;','<').replace('&gt;','>').replace('&amp;','&')
        m[name]=exp
    return m

def expand(text, legend, depth=0):
    if depth>40: return text
    # replace abbreviation names (word boundaries; names are like SI1, EX2, KD19, AM1, H1, PM3, ID1, UN1, KG1...)
    def repl(mo):
        n=mo.group(0)
        if n in legend: return '('+expand(legend[n], legend, depth+1)+')' if False else expand(legend[n], legend, depth+1)
        return n
    # names: uppercase letters (1-2) optionally with digits, or bare number tokens
    return re.sub(r'\b([A-Z]{1,4}\d+|\b\d+\b)\b', repl, text)

expl=0; unexpl=0; ex=[]
files=sorted(glob.glob(CORP+"/*.dot"))
for f in files:
    dot=open(f).read()
    leg=None
    for line in dot.splitlines():
        if 'shape="record"' not in line: continue
        m=re.search(r'label="(.*)",fillcolor', line)
        if not m: continue
        groups=parse_record(m.group(1))
        if groups is None: continue
        for cells in groups:
            if is_info(cells): continue
            flats=[flat_width(c) for c in cells]
            T=sum(flats)
            for k,c in enumerate(cells):
                actual='\\l' in c
                fw=flats[k]; b0=max(87-(T-fw),20); p0=fw>b0
                if (not p0) and actual:  # false_neg
                    if leg is None: leg=legend_map(dot)
                    txt,lost=flat_text(c)
                    uexp=expand(txt, leg)
                    uw=len(uexp)+lost
                    # unabbreviated budget: siblings also expanded
                    sib_uw=0
                    for j,cc in enumerate(cells):
                        if j==k: continue
                        t2,l2=flat_text(cc); sib_uw+=len(expand(t2,leg))+l2
                    ub=max(87-sib_uw,20)
                    explained = uw>ub
                    if explained: expl+=1
                    else:
                        unexpl+=1
                        if len(ex)<15: ex.append((f.split('/')[-1],T,fw,uw,ub,c[:55]))
print(f"false_neg explained by UNABBREVIATED width > unabbrev budget: {expl}")
print(f"false_neg NOT explained: {unexpl}")
print("unexplained examples (T_disp, flat_disp, flat_unabbrev, unabbrev_budget, cell):")
for e in ex: print("  ",e)
