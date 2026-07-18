#!/usr/bin/env python3
"""Round-13 census: classify every ported edge endpoint by port-class.

Info cell is the one whose text starts with '#' (a temporal var `#t : rule`);
prem/concl by column position relative to the info column.
"""
import os, re, collections

CORPUS = "/home/kamilner/tamarin-cleanroom/graphdot/oracle/dot_corpus"

rec_re = re.compile(r'^(n\d+)\[shape="record",label="(.*)",fillcolor=', re.S)
edge_re = re.compile(r'^(n\d+)(?::(n\d+))? -> (n\d+)(?::(n\d+))?(\[[^\]]*\])?;$')

def parse_record(label):
    """Return dict port_id -> class ('prem'|'info'|'concl').
    Walk top-level columns (depth-1 '|' splits); a port's cell text is the
    chars after '<nX>'; info column = the one whose first cell starts '#'."""
    depth = 0
    col_idx = 0
    # (col_idx, port, first_text_char)
    ports = []
    i, n = 0, len(label)
    while i < n:
        c = label[i]
        if c == '\\':
            i += 2; continue
        if c == '{':
            depth += 1; i += 1; continue
        if c == '}':
            depth -= 1; i += 1; continue
        if c == '|' and depth == 1:
            col_idx += 1; i += 1; continue
        if c == '<':
            j = label.find('>', i)
            tok = label[i+1:j]
            k = j + 1
            while k < n and label[k] == ' ':
                k += 1
            first = label[k] if k < n else ''
            if re.fullmatch(r'n\d+', tok):
                ports.append((col_idx, tok, first))
            i = j + 1; continue
        i += 1
    # info column = min col_idx whose a cell first char == '#'
    info_col = None
    for col_idx, tok, first in ports:
        if first == '#':
            info_col = col_idx
            break
    out = {}
    for col_idx, tok, first in ports:
        if info_col is None:
            out[tok] = 'unk'
        elif col_idx == info_col:
            out[tok] = 'info'
        elif col_idx < info_col:
            out[tok] = 'prem'
        else:
            out[tok] = 'concl'
    return out

pos_style_class = collections.Counter()
files_with_info_src = 0
total_files = 0
info_src_edges = 0
info_dst_edges = 0
info_as_target_examples = []
info_as_source_styles = collections.Counter()
# also: for info-source edges, what is the target port class?
info_src_target = collections.Counter()

for fn in sorted(os.listdir(CORPUS)):
    if not fn.endswith('.dot'):
        continue
    total_files += 1
    with open(os.path.join(CORPUS, fn), encoding='utf-8') as f:
        lines = f.read().splitlines()
    portclass = {}
    for ln in lines:
        m = rec_re.match(ln)
        if m:
            portclass.update(parse_record(m.group(2)))
    this_info_src = False
    for ln in lines:
        m = edge_re.match(ln.strip())
        if not m:
            continue
        srcn, srcp, dstn, dstp, attrs = m.groups()
        attrs = attrs or '(none)'
        scl = portclass.get(srcp, 'noRecPort') if srcp else 'NOPORT'
        dcl = portclass.get(dstp, 'noRecPort') if dstp else 'NOPORT'
        pos_style_class[('src', attrs, scl)] += 1
        pos_style_class[('dst', attrs, dcl)] += 1
        if scl == 'info':
            info_src_edges += 1
            this_info_src = True
            info_as_source_styles[attrs] += 1
            info_src_target[(attrs, dcl)] += 1
        if dcl == 'info':
            info_dst_edges += 1
            if len(info_as_target_examples) < 15:
                info_as_target_examples.append((fn, ln.strip()))
    if this_info_src:
        files_with_info_src += 1

print("TOTAL FILES:", total_files)
print("FILES WITH >=1 info-source edge:", files_with_info_src,
      f"({100.0*files_with_info_src/total_files:.1f}%)")
print("INFO-SOURCE edges total:", info_src_edges)
print("INFO-TARGET edges total:", info_dst_edges)
print()
print("=== info-as-SOURCE by style ===")
for a, c in info_as_source_styles.most_common():
    print(f"{c:8d}  {a}")
print()
print("=== info-SOURCE edges: target port-class by style ===")
for (a, dcl), c in info_src_target.most_common():
    print(f"{c:8d}  style={a:34s} target={dcl}")
print()
print("=== info-as-TARGET examples (should be empty if source-only) ===")
for fn, ln in info_as_target_examples:
    print(fn, ln)
print()
print("=== FULL (position, style, portclass) census ===")
for (pos, style, cl), c in sorted(pos_style_class.items(), key=lambda x: -x[1]):
    print(f"{c:8d}  {pos:3s} {cl:9s} {style}")
