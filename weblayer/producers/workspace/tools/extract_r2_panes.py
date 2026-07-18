#!/usr/bin/env python3
"""Materialize the west (proof-script) pane inner HTML of every overview
capture across the 81 manifests (captured OUTPUT) as R2 byte targets.

Writes workspace/r2_panes/<label>__<slug>.pane — the bytes between
`<div class="monospace" id="proof">` and the next `</div>` (the pane never
contains a nested div; asserted). Also usable on a LIVE overview page passed
as a file argument: writes <name>.pane next to it.
"""
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, "..", "..", "oracle", "captured_responses")
OUT = os.path.join(HERE, "..", "r2_panes")

MARK = '<div class="monospace" id="proof">'


def pane_of(body):
    start = body.index(MARK) + len(MARK)
    end = body.index("</div>", start)
    inner = body[start:end]
    assert "<div" not in inner
    return inner


def slug(url):
    return re.sub(r"[^A-Za-z0-9]+", "_", re.sub(r"/thy/(trace|equiv)/[^/]+/", "", url)).strip("_")


def main():
    if len(sys.argv) > 1:
        # live mode: slice one saved overview page
        src = sys.argv[1]
        body = open(src).read()
        out = os.path.splitext(src)[0] + ".pane"
        open(out, "w").write(pane_of(body))
        print("wrote", out)
        return
    os.makedirs(OUT, exist_ok=True)
    n = 0
    for p in sorted(glob.glob(os.path.join(CORPUS, "*.hs.json"))):
        d = json.load(open(p))
        label = os.path.basename(p).split(".")[0][:8]
        for url, r in d.get("manifest", {}).items():
            if r.get("kind") != "html" or "/overview/" not in url:
                continue
            base = "%s__%s" % (label, slug(url))
            open(os.path.join(OUT, base + ".pane"), "w").write(pane_of(r["body"]))
            n += 1
    print("wrote %d panes to %s" % (n, OUT))


if __name__ == "__main__":
    main()
