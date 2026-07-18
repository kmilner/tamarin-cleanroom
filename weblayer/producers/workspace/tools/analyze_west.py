#!/usr/bin/env python3
"""Corpus-wide analysis of the west (proof-script) pane across ALL overview
captures in the 81 manifests (captured OUTPUT only).

Slices each pane into its logical lines and inventories the element grammar:
frame (header/end/final byte), item-line shapes, add-link shapes, lemma
declaration forms (name/attributes/quantifier layout), the inline-vs-vertical
formula rule, proved-header status wrappers, and the proof-line shapes kept
opaque by R2.
"""
import collections
import glob
import html
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, "..", "..", "oracle", "captured_responses")

PANE = re.compile(r'<div class="monospace" id="proof">(.*?)</div>', re.S)
TAG = re.compile(r"<[^>]+>")


def visible_len(line):
    """Visible character count: strip tags, decode entities."""
    return len(html.unescape(TAG.sub("", line)))


def panes():
    for p in sorted(glob.glob(os.path.join(CORPUS, "*.hs.json"))):
        d = json.load(open(p))
        for url, r in d.get("manifest", {}).items():
            if r.get("kind") != "html" or "/overview/" not in url:
                continue
            m = PANE.search(r["body"])
            assert m, (p, url)
            yield os.path.basename(p), url, m.group(1)


def main():
    n = 0
    frame_bad = []
    item_shapes = collections.Counter()
    decl_shapes = collections.Counter()
    quant_forms = collections.Counter()
    header_wrappers = collections.Counter()
    inline_lens = []
    vertical_single = []          # single-line formulas NOT inlined
    vertical_first_lens = []
    proofline_heads = collections.Counter()
    oddities = collections.Counter()
    for fname, url, pane in panes():
        n += 1
        assert "<div" not in pane, (fname, url)
        if not pane.endswith("<br/>\n "):
            frame_bad.append((fname, url, pane[-30:]))
        lines = pane[: -1].split("<br/>\n")
        assert lines[-1] == "", (fname, url)
        lines = lines[:-1]
        # frame
        if not (lines[0].startswith('<span class="hl_keyword">theory</span> ')
                and lines[0].endswith('<span class="hl_keyword">begin</span>')):
            oddities["header"] += 1
        if lines[-1] != '<span class="hl_keyword">end</span>':
            oddities["end:" + lines[-1][:40]] += 1
        i = 1
        # items until the add-first link
        while i < len(lines):
            if 'class="internal-link add"' in lines[i]:
                break
            if lines[i] == "":
                i += 1
                continue
            m = re.match(
                r'<a class="internal-link" href="/thy/trace/(\d+)/main/([^"]*)">'
                r"<strong>(.*?)</strong> (.*)</a>$",
                lines[i],
            )
            if m:
                item_shapes[(m.group(2), m.group(3), "ann" if m.group(4) else "empty")] += 1
            else:
                item_shapes[("UNMATCHED", lines[i][:60], "")] += 1
            i += 1
        # lemma blocks: find each decl line
        for j, ln in enumerate(lines):
            m = re.match(
                r'(?:<span class="(?P<wrap>[\w-]+)">)?'
                r'<span class="hl_keyword">lemma</span> (?P<rest>.*):$',
                ln,
            )
            if m:
                header_wrappers[m.group("wrap") or "NONE"] += 1
                rest = m.group("rest")
                am = re.match(r"^([A-Za-z0-9_.]+)(.*)$", rest)
                if am:
                    decl_shapes[am.group(2)] += 1
                else:
                    decl_shapes["UNMATCHED:" + rest[:40]] += 1
                # quantifier line
                q = lines[j + 1]
                qm = re.match(r"^  (all-traces|exists-trace)( (.*))?$", q)
                if not qm:
                    quant_forms["UNMATCHED:" + q[:50]] += 1
                    continue
                quant_forms[(qm.group(1), "inline" if qm.group(2) else "vertical")] += 1
                if qm.group(2):
                    inline_lens.append(visible_len(q))
                else:
                    # count formula lines until the edit line
                    k = j + 2
                    while k < len(lines) and 'class="internal-link edit"' not in lines[k]:
                        k += 1
                    nlines = k - (j + 2)
                    vertical_first_lens.append(visible_len(lines[j + 2]) + visible_len(q))
                    if nlines == 1:
                        vertical_single.append((fname, url, visible_len(q), visible_len(lines[j + 2]), lines[j+1], lines[j+2][:90]))
        # proof-line heads (lines between edit-line and blank)
        for ln in lines:
            if re.match(r"^(&nbsp;)*<a class=\"internal-link proof-step", ln):
                proofline_heads["step-nb"] += 1
            elif re.match(r"^(&nbsp;)*<span class=\"[\w-]+\"><span class=\"hl_keyword\">(case|next|qed|by)</span>", ln):
                proofline_heads["kw"] += 1
    print("panes:", n, "frame_bad:", frame_bad[:3], "oddities:", dict(oddities))
    print("\nitem shapes:")
    for k, c in sorted(item_shapes.items()):
        print("  %6d %s" % (c, k))
    print("\ndecl attribute tails:")
    for k, c in sorted(decl_shapes.items(), key=lambda x: -x[1])[:25]:
        print("  %6d %r" % (c, k))
    print("\nquantifier forms:", dict(quant_forms))
    print("header wrappers:", dict(header_wrappers))
    if inline_lens:
        print("inline total-line visible len: max", max(inline_lens))
    if vertical_first_lens:
        print("vertical q+first-line visible len: min", min(vertical_first_lens))
    print("vertical single-line formulas:", len(vertical_single))
    for v in vertical_single[:5]:
        print("   ", v)
    print("proofline heads:", dict(proofline_heads))


if __name__ == "__main__":
    sys.exit(main())
