#!/usr/bin/env python3
"""Round-7 probe theory generator + DOT cell parser (clean-room, black-box).

We build one-rule theories whose single graph node is a record with a
controlled conclusion (or premise) group, autoprove an exists-trace lemma per
rule, fetch the proof-state graph DOT, and read back the wrapped layout of a
target cell.  All widths are engineered from distinct <10-char / occ-1 atoms so
node abbreviation (len>=10 & occ>=2) never fires and the flat widths are exact.
"""
import re, sys

# ---- fact builders (flat widths are exact & abbreviation-proof) --------------

def tup(n, start=1):
    """A tuple fact body of n distinct 5-char public constants 'eNN'."""
    return "<" + ", ".join(f"'e{start+i:02d}'" for i in range(n)) + ">"

def big(n, name="Big", start=1):
    """Wrapping-tuple cell: NAME(<'e01',...,'eNN'>). flat width computed by measure."""
    return f"{name}({tup(n, start)})"

def atom_fact(flat, name, fill='b'):
    """A single-atom fact NAME('bbb..') whose flat rendering is exactly `flat`.
    flat = len(NAME)+2 (for '( ') + (L+2) quotes/atom + 2 (' )'); L = atom chars.
    `fill` MUST differ between sibling facts of equal length, else the >=10-char
    atom repeats (occ 2) and gets ABBREVIATED, corrupting the flat width."""
    L = flat - (len(name) + 2) - 2 - 2
    if L < 1:
        raise ValueError(f"flat {flat} too small for name {name}")
    return f"{name}('{fill*L}')"

def flat_of(term_str):
    """Flat visual width of a rendered fact string == its char count."""
    return len(term_str)

# ---- theory assembly ---------------------------------------------------------

def build_theory(name, rules):
    """rules: list of (lemma_name, [conclusion fact strings]).
    Each becomes  rule R: [Fr(~s)] --[R()]-> [concls]  + exists-trace lemma."""
    out = [f"theory {name}", "begin", ""]
    lem = []
    for rn, concls in rules:
        body = ", ".join(concls)
        out.append(f"rule {rn}:\n  [ Fr(~s) ] --[ {rn}() ]-> [ {body} ]\n")
        lem.append(f'lemma l_{rn}: exists-trace "Ex #i. {rn}() @ #i"')
    out += lem + ["", "end", ""]
    return "\n".join(out)

# ---- DOT record-cell parser --------------------------------------------------

def find_record_label(dot):
    m = re.search(r'shape="record",label="(.*?)",fillcolor=', dot)
    return m.group(1) if m else None

def split_groups(label):
    """label = {{g0}|{g1}|{g2}} -> list of group inner strings (cells joined by |)."""
    assert label.startswith("{{") and label.endswith("}}"), label[:40]
    inner = label[2:-2]
    # split on }|{ at top level
    return re.split(r'\}\|\{', inner)

def split_cells(group):
    """group inner = <p0> cell0|<p1> cell1|... -> list of raw cell texts (port stripped)."""
    # cells separated by | that are NOT escaped (\|) and not inside the cell text.
    # Ports look like <nK> ; split on '|<n' but keep structure.
    parts = re.split(r'\|(?=<n\d+> )', group)
    cells = []
    for p in parts:
        m = re.match(r'<n\d+> (.*)$', p, re.S)
        cells.append(m.group(1) if m else p)
    return cells

def unescape_cell(cell):
    """Undo record escaping for width/element analysis: \< \> \{ \} \| -> literal."""
    return re.sub(r'\\([<>{}|])', r'\1', cell)

def physical_lines(cell):
    """Split a wrapped cell into physical lines. Segments end with \\l; continuation
    lines are prefixed by &nbsp; runs. Returns list of (indent, text) unescaped."""
    if '\\l' not in cell:
        return [(0, unescape_cell(cell))]
    segs = cell.split('\\l')
    if segs and segs[-1] == '':
        segs = segs[:-1]
    out = []
    for s in segs:
        m = re.match(r'((?:&nbsp;)*)(.*)$', s, re.S)
        ind = len(m.group(1)) // len('&nbsp;')
        out.append((ind, unescape_cell(m.group(2))))
    return out

def line0_tuple_elements(cell):
    """For a wrapping tuple-fact cell like NAME( <'e01', 'e02', ... >, count the
    number of tuple elements on physical line 0."""
    lines = physical_lines(cell)
    l0 = lines[0][1]
    # strip leading NAME( < ; count 'eNN' tokens on the line
    return len(re.findall(r"'e\d+'", l0)), l0, lines
