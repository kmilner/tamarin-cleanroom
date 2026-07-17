#!/usr/bin/env python3
"""Python port of the crate's greedy fill (render.rs run_layout/layout_fact/
layout_tuple) so we can measure, over the corpus, how well a given per-cell
budget reproduces the ACTUAL line-0 break of a wrapping cell."""
import re

def width(s): return len(s)

def split_top_commas(s):
    out=[]; cur=[]; depth=0; q=False; i=0
    while i < len(s):
        c=s[i]
        if q:
            cur.append(c)
            if c=="'": q=False
            i+=1; continue
        if c=="'": q=True; cur.append(c); i+=1; continue
        if c in '(<[': depth+=1; cur.append(c); i+=1; continue
        if c in ')>]': depth-=1; cur.append(c); i+=1; continue
        if c==',' and depth==0 and i+1<len(s) and s[i+1]==' ':
            out.append(''.join(cur)); cur=[]; i+=2; continue
        cur.append(c); i+=1
    out.append(''.join(cur)); return out

def run_layout(open_text, start_col, units, budget, drop_break_space):
    # units: list of (sep, text, indent, hard)
    lines=[]; cur=open_text; col=start_col; line_indent=0
    for (sep,text,indent,hard) in units:
        cur+=sep; col+=len(sep); w=len(text)
        if hard or col+w>budget:
            if drop_break_space and cur.endswith(' '): cur=cur[:-1]
            lines.append((line_indent,cur)); line_indent=indent; cur=text; col=indent+w
        else:
            cur+=text; col+=w
    lines.append((line_indent,cur)); return lines

def is_tuple(s): return s.startswith('<') and s.endswith('>')

def layout_tuple(inner, base_col, budget):
    elems=split_top_commas(inner); open_col=base_col+1
    units=[]
    for i,e in enumerate(elems):
        units.append(("" if i==0 else ", ", e, open_col, False))
    units.append(("", ">", base_col, False))
    return run_layout("<", open_col, units, budget, False)

def layout_fact(flat, budget):
    if width(flat)<=budget: return [(0,flat)]
    lp=flat.find("( ")
    if lp<0 or not flat.endswith(" )"): return [(0,flat)]
    open_str=flat[:lp+2]; content=flat[lp+2:len(flat)-2]; open_col=len(open_str)
    args=split_top_commas(content)
    if len(args)==1 and is_tuple(args[0]):
        inner=args[0][1:-1]
        tl=layout_tuple(inner, open_col, budget)
        lines=[(0, open_str+tl[0][1])]+tl[1:]+[(0,")")]
        return lines
    units=[]
    for i,a in enumerate(args):
        units.append(("" if i==0 else ", ", a, open_col, False))
    units.append(("", ")", 0, True))
    return run_layout(open_str, open_col, units, budget, True)

def layout_info(flat, budget):
    lb=flat.find('[')
    if lb<0 or not flat.endswith(']'): return [(0,flat)]
    open_str=flat[:lb+1]; content=flat[lb+1:len(flat)-1]; open_col=len(open_str)
    acts=split_top_commas(content)
    if len(acts)<=1 and width(flat)<=budget: return [(0,flat)]
    units=[]
    for i,a in enumerate(acts):
        text=(a+",") if i+1<len(acts) else (a+"]")
        units.append(("", text, open_col, i>0))
    return run_layout(open_str, open_col, units, budget, False)

def layout_cell(flat, budget):
    if flat.startswith('#') and '[' in flat: return layout_info(flat,budget)
    return layout_fact(flat,budget)

def cell_occ(flat, budget):
    return max(ind+len(t) for ind,t in layout_cell(flat,budget))
