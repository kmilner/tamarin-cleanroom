#!/usr/bin/env python3
import sys, re, os
sys.path.insert(0, '/home/kamilner/tamarin-cleanroom/graphdot/workspace/r7')
from genlib import find_record_label, split_groups, split_cells, physical_lines

OUT = sys.argv[1]

def cell_named(dot, pfx):
    label = find_record_label(dot)
    for g in split_groups(label):
        for c in split_cells(g):
            if c.lstrip().startswith(pfx):
                return c
    return None

# D4: occ of wrapped single atom
print("=== D4: occ (widest line) of a forced-wrapped single-atom fact ===")
for name in sorted(os.listdir(OUT)):
    m = re.match(r'l_D4_f(\d+)\.dot', name)
    if not m: continue
    f = int(m.group(1))
    c = cell_named(open(os.path.join(OUT, name)).read(), "Wp(")
    lines = physical_lines(c)
    widths = [len(t) for _, t in lines]
    occ = max(widths)
    print(f"  Wp flat={f}: lines widths={widths}  occ(max)={occ}  flat-2={f-2}  wrapped={'\\l' in c}")

# D2/D3: trigger boundary
conf = {}
for name in sorted(os.listdir(OUT)):
    m = re.match(r'l_(D[23]_a\d+)_p(\d+)\.dot', name)
    if not m: continue
    key, p = m.group(1), int(m.group(2))
    c = cell_named(open(os.path.join(OUT, name)).read(), "Ta(")
    conf.setdefault(key, []).append((p, '\\l' in c))

print("\n=== D2/D3: Ta trigger boundary ===")
print(f"{'config':12s} {'fitmax':>6s} {'wrapmin':>7s}  {'budget':>6s} notes")
for key in sorted(conf):
    lst = sorted(conf[key])
    fits = [p for p, w in lst if not w]; wr = [p for p, w in lst if w]
    fm = max(fits) if fits else None
    wm = min(wr) if wr else None
    ma = re.match(r'D(\d)_a(\d+)', key); kind = ma.group(1); a = int(ma.group(2))
    if kind == '2':
        flatsum = max(87 - a - 10, 20); model = max(87 - (a-2) - 10, 20)
    else:
        flatsum = max(87 - 2*a, 20); model = max(87 - 2*(a-2), 20)
    print(f"{key:12s} {str(fm):>6s} {str(wm):>7s}  {str(fm):>6s} flatsum={flatsum} occ-model={model}")
