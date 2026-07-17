#!/usr/bin/env python3
import sys, re, os
sys.path.insert(0,'/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import find_record_label, split_groups, split_cells

OUT=sys.argv[1]
def acell(dot):
    for g in split_groups(find_record_label(dot)):
        for c in split_cells(g):
            if c.lstrip().startswith("Aa("): return c
    return None

conf={}
for name in sorted(os.listdir(OUT)):
    if not name.endswith('.dot'): continue
    tag=name[2:-4]
    m=re.match(r'(G1_q\d+_[AB]{2}|G3_[ABC]{3})_p(\d+)$', tag)
    if not m: continue
    key,p=m.group(1),int(m.group(2))
    c=acell(open(os.path.join(OUT,name)).read())
    if c is None: continue
    conf.setdefault(key,[]).append((p,'\\l' in c))

print(f"{'config':16s} {'A-fitmax':>8s} {'A-wrapmin':>9s}")
for key in sorted(conf):
    lst=sorted(conf[key]); fits=[p for p,w in lst if not w]; wr=[p for p,w in lst if w]
    print(f"{key:16s} {str(max(fits) if fits else None):>8s} {str(min(wr) if wr else None):>9s}")
