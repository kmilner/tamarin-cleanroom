#!/usr/bin/env python3
import sys, re, os
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import find_record_label, split_groups, split_cells

OUT = sys.argv[1]

def tacell(dot):
    for g in split_groups(find_record_label(dot)):
        for c in split_cells(g):
            if c.lstrip().startswith("Ta("):
                return c
    return None

def has_legend(dot):
    return 'shape="plain"' in dot  # abbreviation legend node present => contamination

conf = {}
contaminated = set()
for name in sorted(os.listdir(OUT)):
    if not name.endswith(".dot"): continue
    tag = name[2:-4]
    m = re.match(r'(E_\w+?)_p(\d+)$', tag)
    if not m: continue
    key, p = m.group(1), int(m.group(2))
    dot = open(os.path.join(OUT, name)).read()
    if has_legend(dot):
        contaminated.add(key)
    c = tacell(dot)
    if c is None: continue
    conf.setdefault(key, []).append((p, '\\l' in c))

print(f"{'config':14s} {'fitmax':>6s} {'wrapmin':>7s} {'budget':>6s}  note")
for key in sorted(conf):
    lst = sorted(conf[key]); fits=[p for p,w in lst if not w]; wr=[p for p,w in lst if w]
    fm=max(fits) if fits else None; wm=min(wr) if wr else None
    clean = "" if (fm is None or wm is None or wm==fm+1) else " NOTMONO"
    con = " CONTAM" if key in contaminated else ""
    print(f"{key:14s} {str(fm):>6s} {str(wm):>7s} {str(fm):>6s} {clean}{con}")
