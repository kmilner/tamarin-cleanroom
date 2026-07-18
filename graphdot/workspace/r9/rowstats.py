#!/usr/bin/env python3
"""Mine observed row-occupancy invariants directly from corpus DOT records.

For each prem/concl group: per cell, observed max physical line width mw
(&nbsp; = 1 col), height (line count), flat width. Look at Sum(mw) for rows
with >=1 wrapping cell, split by composition.
"""
import sys, os, re
from collections import Counter, defaultdict

def split_top(s, sep):
    parts, cur, depth, i = [], [], 0, 0
    while i < len(s):
        c = s[i]
        if c == "\\" and i + 1 < len(s):
            cur.append(s[i:i+2]); i += 2; continue
        if c == "{": depth += 1; cur.append(c)
        elif c == "}": depth -= 1; cur.append(c)
        elif c == sep and depth == 0:
            parts.append("".join(cur)); cur = []
        else: cur.append(c)
        i += 1
    parts.append("".join(cur))
    return parts

def unescape(t):
    for a, b in (("\\<","<"),("\\>",">"),("\\{","{"),("\\}","}"),("\\|","|")):
        t = t.replace(a, b)
    return t

def cell_lines(body):
    """Return list of (physical line col-width) for a cell body."""
    segs = body.split("\\l")
    if segs and segs[-1] == "":
        segs = segs[:-1]
    out = []
    for s in segs:
        ind = 0
        while s.startswith("&nbsp;"):
            ind += 1; s = s[6:]
        out.append(ind + len(unescape(s)))
    return out

def flat_width(b):
    lost = b.count(",\\l") + b.count("\\l)")
    return len(unescape(b.replace("\\l", "").replace("&nbsp;", ""))) + lost

def parse_file(path):
    with open(path) as f:
        dot = f.read()
    for line in dot.splitlines():
        if 'shape="record"' not in line: continue
        ls = line.find('label="')
        if ls < 0: continue
        after = line[ls+7:]
        le = after.find('",fillcolor')
        if le < 0: continue
        label = after[:le]
        if not (label.startswith("{") and label.endswith("}")): continue
        groups = []
        okrec = True
        for g in split_top(label[1:-1], "|"):
            g = g.strip()
            if not (g.startswith("{") and g.endswith("}")): okrec = False; break
            cells = []
            for c in split_top(g[1:-1], "|"):
                m = re.match(r"<n(\d+)> (.*)$", c, re.S)
                if not m: cells.append(None); continue
                cells.append(m.group(2))
            groups.append(cells)
        if not okrec: continue
        yield groups

def is_info(b):
    return b is not None and b.startswith("#") and " : " in b

def main():
    corpus = sys.argv[1]
    files = sorted(os.listdir(corpus))
    step = max(1, len(files) // 2000)
    sum_hist = Counter()          # Sum(mw) over rows with any wrap
    sum_hist_allwrap = Counter()  # rows where every cell wraps
    ex_by_sum = defaultdict(list)
    n_hist = Counter()
    for fi, fn in enumerate(files):
        if fi % step or not fn.endswith(".dot"): continue
        for groups in parse_file(os.path.join(corpus, fn)):
            info_idx = None
            for gi, cells in enumerate(groups):
                if len(cells) == 1 and is_info(cells[0]):
                    info_idx = gi
            if info_idx is None: continue
            for gi, cells in enumerate(groups):
                if gi == info_idx or any(c is None for c in cells): continue
                if any(is_info(c) for c in cells): continue
                wraps = ["\\l" in c for c in cells]
                if not any(wraps): continue
                mws = [max(cell_lines(c)) for c in cells]
                hs = [len(cell_lines(c)) for c in cells]
                flats = [flat_width(c) for c in cells]
                S = sum(mws)
                sum_hist[S] += 1
                n_hist[len(cells)] += 1
                if all(wraps):
                    sum_hist_allwrap[S] += 1
                if len(ex_by_sum[S]) < 3:
                    ex_by_sum[S].append((fn, flats, mws, hs))
    tot = sum(sum_hist.values())
    print(f"rows with any wrapping cell: {tot}")
    print("Sum(mw) histogram (top 30):")
    for s, c in sum_hist.most_common(30):
        print(f"  S={s:4d}: {c:6d}  ({100*c/tot:.1f}%)")
    print("cumulative: S<=87:", sum(c for s, c in sum_hist.items() if s <= 87),
          " S in 80..87:", sum(c for s, c in sum_hist.items() if 80 <= s <= 87),
          " S>87:", sum(c for s, c in sum_hist.items() if s > 87))
    ta = sum(sum_hist_allwrap.values())
    print(f"\nall-wrap rows: {ta}; S<=87: {sum(c for s,c in sum_hist_allwrap.items() if s<=87)}")
    print("\nexamples with S>95:")
    shown = 0
    for s in sorted(ex_by_sum, reverse=True):
        if s <= 95: break
        for e in ex_by_sum[s]:
            print(f"  S={s}", e)
            shown += 1
            if shown >= 15: break
        if shown >= 15: break

if __name__ == "__main__":
    main()
