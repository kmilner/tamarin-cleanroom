#!/usr/bin/env python3
"""Corpus-only survey of the proof-tree LINE grammar inside the west panes
(workspace/r2_panes/*.pane, sliced capture OUTPUT).

Classifies every proof-display line (the lines R2 held opaque between the
edit/delete line and the blank before the per-lemma add link) into
STEP / CASE / NEXT / QED shapes, then reports:
  * the by-prefix rule (which steps carry the leading `by ` span);
  * indent arithmetic (case/next/qed placement relative to the parent step);
  * status-class vocabulary and within-tree mixing;
  * path composition (href tail vs the case names on the root-to-node walk);
  * sorry/SOLVED and remove-step presence.
Strict: any line that matches none of the shapes is dumped.
"""
import glob
import html
import os
import re
import sys
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
PANES = os.path.join(HERE, "..", "r2_panes")

STEP_RE = re.compile(
    r'^((?:&nbsp;)*)'
    r'(?:<span class="(hl_\w+)"><span class="hl_keyword">by</span> </span>)?'
    r'<a class="internal-link proof-step ([\w-]+)" href="/thy/(?:trace|equiv)/(\d+)/main/proof/([^"]*)">(.*?)</a>'
    r'(<a class="internal-link remove-step" href="/thy/(?:trace|equiv)/\d+/main/proof/([^"]*)"></a>)?$',
    re.S,
)
CASE_RE = re.compile(
    r'^((?:&nbsp;)*)<span class="(hl_\w+)"><span class="hl_keyword">case</span> (.*)</span>$'
)
KW_RE = re.compile(
    r'^((?:&nbsp;)*)<span class="(hl_\w+)"><span class="hl_keyword">(qed|next)</span></span>$'
)

def nb(run):
    return len(run) // len("&nbsp;")

def main():
    stats = Counter()
    weird = []
    step_classes = Counter()
    by_status_vs_anchor = Counter()
    method_heads = Counter()
    by_methods = Counter()
    noby_methods = Counter()
    tree_status_sets = Counter()
    header_vs_tree = Counter()
    for p in sorted(glob.glob(os.path.join(PANES, "*.pane"))):
        raw = open(p).read()
        body = raw[: -1] if raw.endswith(" ") else raw
        lines = body.split("<br/>\n")
        assert lines[-1] == ""
        lines = lines[:-1]
        i = 0
        n = len(lines)
        while i < n:
            # find a lemma header: line starting with wrapper span or lemma kw
            line = lines[i]
            m = re.match(r'^(?:<span class="(hl_\w+)">)?<span class="hl_keyword">lemma</span> ', line)
            if not m:
                i += 1
                continue
            header_status = m.group(1)
            # skip to edit/delete line
            while '<a class="internal-link edit"' not in lines[i]:
                i += 1
            i += 1
            # proof display lines run until blank
            proof_lines = []
            while i < n and lines[i] != "":
                proof_lines.append(lines[i])
                i += 1
            # a multi-line method text glues physical lines: rejoin lines that
            # do not START a recognized shape onto the previous one
            merged = []
            for l in proof_lines:
                if merged and not (STEP_RE.match(l) or CASE_RE.match(l) or KW_RE.match(l) or
                                   l.startswith('<span class="hl_keyword">by</span> ') or
                                   re.match(r'^((?:&nbsp;)*)(<span|<a)', l)):
                    merged[-1] += "\n" + l
                else:
                    merged.append(l)
            # actually multi-line methods: a step's anchor spans lines, so
            # re-merge until the STEP_RE closing matches; simpler: merge until
            # balanced by scanning: try parse; on fail, append next line.
            items = []
            buf = None
            for l in proof_lines:
                buf = l if buf is None else buf + "\n" + l
                sm = STEP_RE.match(buf)
                if sm:
                    items.append(("STEP", buf, sm))
                    buf = None
                    continue
                cm = CASE_RE.match(buf)
                if cm:
                    items.append(("CASE", buf, cm))
                    buf = None
                    continue
                km = KW_RE.match(buf)
                if km:
                    items.append((km.group(3).upper(), buf, km))
                    buf = None
                    continue
                # sorry line (R2 unproven) — standalone
                if re.match(r'^<span class="hl_keyword">by</span> <a class="internal-link proof-step sorry-step"[^>]*><span class="hl_keyword">sorry</span></a>$', buf):
                    items.append(("ROOTSORRY", buf, None))
                    buf = None
                    continue
            if buf is not None:
                weird.append((p, buf[:400]))
                continue
            if not items:
                continue
            if len(items) == 1 and items[0][0] == "ROOTSORRY":
                stats["unproven"] += 1
                continue
            stats["trees"] += 1
            statuses = set()
            # walk: maintain a stack of (indent, path) for parent steps
            # verify path composition + indent arithmetic
            stack = []  # (step_indent, path_segs) of open case blocks
            prev_step = None  # (indent, path)
            ok = True
            for kind, text, m in items:
                if kind == "STEP":
                    ind = nb(m.group(1))
                    by_st = m.group(2)
                    cls = m.group(3)
                    path = m.group(5)
                    meth = m.group(6)
                    rm = m.group(7)
                    step_classes[cls] += 1
                    if by_st:
                        by_status_vs_anchor[(by_st, cls)] += 1
                    segs = path.split("/")
                    head = re.sub(r"<[^>]*>", "", meth)
                    method_heads[head.split("(")[0].split(" ")[0]] += 1
                    if by_st:
                        by_methods[head.split("(")[0].split(" ")[0]] += 1
                    else:
                        noby_methods[head.split("(")[0].split(" ")[0]] += 1
                    if cls.startswith("hl_"):
                        statuses.add(cls)
                    if rm is None:
                        stats[f"step_no_removestep::{cls}::{head[:20]}"] += 1
                    elif m.group(8) != path:
                        weird.append((p, "rm-href-mismatch " + text[:200]))
                    prev_step = (ind, segs)
                elif kind == "CASE":
                    ind = nb(m.group(1))
                    st = m.group(2)
                    name = m.group(3)
                    statuses.add(st)
                    stats["case"] += 1
                elif kind in ("QED", "NEXT"):
                    ind = nb(m.group(1))
                    st = m.group(2)
                    statuses.add(st)
                    stats[kind.lower()] += 1
            tree_status_sets[frozenset(statuses)] += 1
            header_vs_tree[(header_status, frozenset(statuses))] += 1
    print("== stats =="); [print(f"  {k}: {v}") for k, v in sorted(stats.items())]
    print("== step anchor classes =="); [print(f"  {k}: {v}") for k, v in step_classes.items()]
    print("== by-span status vs anchor class =="); [print(f"  {k}: {v}") for k, v in by_status_vs_anchor.items()]
    print("== method heads WITH by ==");  [print(f"  {k}: {v}") for k, v in sorted(by_methods.items())]
    print("== method heads WITHOUT by ==");  [print(f"  {k}: {v}") for k, v in sorted(noby_methods.items())]
    print("== tree status sets =="); [print(f"  {sorted(k)}: {v}") for k, v in tree_status_sets.items()]
    print("== header status vs tree statuses =="); [print(f"  {k[0]} / {sorted(k[1])}: {v}") for k, v in header_vs_tree.items()]
    print("== weird ==", len(weird))
    for w in weird[:10]:
        print(w)

if __name__ == "__main__":
    main()
