#!/usr/bin/env python3
import sys, re, os
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import find_record_label, split_groups, split_cells, physical_lines

OUT = sys.argv[1] if len(sys.argv) > 1 else "A_dots"

def analyze(path):
    dot = open(path).read()
    label = find_record_label(dot)
    groups = split_groups(label)
    # find the group containing the Big( cell
    for g in groups:
        cells = split_cells(g)
        for c in cells:
            if c.lstrip().startswith("Big("):
                lines = physical_lines(c)
                l0 = lines[0][1]
                m = len(re.findall(r"'e\d+'", l0))
                # element geometry: 5-char elems 'eNN', open_col 6, sep 2
                # B in [7M+4, 7M+10]
                lo, hi = 7*m+4, 7*m+10
                # flats of all cells in this group
                flats = []
                for cc in cells:
                    pls = physical_lines(cc)
                    # flat = de-wrapped width: join, but easier: reconstruct.
                    flats.append(cc)
                return m, lo, hi, l0, cells
    return None

for name in sorted(os.listdir(OUT)):
    if not name.endswith(".dot"): continue
    r = analyze(os.path.join(OUT, name))
    if r is None:
        print(f"{name[:-4]:20s} NO-BIG"); continue
    m, lo, hi, l0, cells = r
    tag = name[2:-4]  # strip l_ and .dot
    print(f"{tag:16s} M={m:2d}  budget in [{lo},{hi}]   cells={len(cells)}")
