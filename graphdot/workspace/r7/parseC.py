#!/usr/bin/env python3
"""Per config, find p_fitmax (largest target flat that stays on one line) and
p_wrapmin (smallest that wraps).  budget = p_fitmax."""
import sys, re, os
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import find_record_label, split_groups, split_cells

OUT = sys.argv[1]
conf = {}  # key -> list of (p, wrapped)
for name in sorted(os.listdir(OUT)):
    if not name.endswith(".dot"): continue
    dot = open(os.path.join(OUT, name)).read()
    label = find_record_label(dot)
    tcell = None
    for g in split_groups(label):
        for c in split_cells(g):
            if c.lstrip().startswith("Ta("):
                tcell = c
    if tcell is None: continue
    wrapped = '\\l' in tcell
    tag = name[2:-4]
    m = re.match(r'(C2_q\d+|C3_a\d+_b\d+)_p(\d+)', tag)
    key = m.group(1); p = int(m.group(2))
    conf.setdefault(key, []).append((p, wrapped))

def others_sum(key):
    m2 = re.match(r'C2_q(\d+)', key)
    if m2: return int(m2.group(1))
    m3 = re.match(r'C3_a(\d+)_b(\d+)', key)
    return int(m3.group(1)) + int(m3.group(2))

print(f"{'config':16s} {'fitmax':>6s} {'wrapmin':>7s} {'budget':>6s} {'87-Sig':>7s} {'diff':>4s}")
for key in sorted(conf):
    lst = sorted(conf[key])
    fits = [p for p, w in lst if not w]
    wr = [p for p, w in lst if w]
    fm = max(fits) if fits else None
    wm = min(wr) if wr else None
    sig = others_sum(key)
    fs = max(87 - sig, 20)
    diff = (fm - fs) if fm is not None else None
    clean = "" if (fm is None or wm is None or wm == fm+1) else " NOTCLEAN"
    print(f"{key:16s} {str(fm):>6s} {str(wm):>7s} {str(fm):>6s} {fs:>7d} {str(diff):>4s}{clean}")
