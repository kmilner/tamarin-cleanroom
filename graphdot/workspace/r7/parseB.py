#!/usr/bin/env python3
"""For each Tgt(<PAD,'y'>) graph, report whether 'y' is on physical line 0.
Then per q, B = (max W with 'y' on line0) + 13."""
import sys, re, os
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import find_record_label, split_groups, split_cells, physical_lines

OUT = sys.argv[1]
rows = {}  # q -> list of (W, y_on_line0)
for name in sorted(os.listdir(OUT)):
    if not name.endswith(".dot"): continue
    dot = open(os.path.join(OUT, name)).read()
    label = find_record_label(dot)
    tgtcell = None
    for g in split_groups(label):
        for c in split_cells(g):
            if c.lstrip().startswith("Tgt("):
                tgtcell = c
    if tgtcell is None: continue
    lines = physical_lines(tgtcell)
    y_l0 = "'y'" in lines[0][1]
    tag = name[2:-4]
    m = re.match(r'B[LF]_(?:q(\d+)_)?w(\d+)', tag)
    q = int(m.group(1)) if m.group(1) else 0
    w = int(m.group(2))
    rows.setdefault(q, []).append((w, y_l0))

print("q     Wstay(max W with 'y' on line0)   B=Wstay+13   87-q")
for q in sorted(rows):
    lst = sorted(rows[q])
    stay = [w for w, y in lst if y]
    peel = [w for w, y in lst if not y]
    wstay = max(stay) if stay else None
    wpeel = min(peel) if peel else None
    B = wstay + 13 if wstay is not None else None
    flag = "" if (wpeel is None or wstay is None or wpeel == wstay+1) else "  <TRANSITION-NOT-CLEAN>"
    print(f"{q:3d}   Wstay={wstay} Wpeel={wpeel}   B={B}   87-q={87-q}{flag}")
