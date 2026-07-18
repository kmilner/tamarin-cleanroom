#!/usr/bin/env python3
"""Materialize the distinct `main/*` wildcard tails of every href in the
captured corpus (captured OUTPUT) as R5 byte targets.

Writes workspace/r5_tails/tails.txt: one line per DISTINCT tail (the part after
`/thy/<kind>/<idx>/main/`), prefixed by its corpus occurrence count, sorted.
The R5 corpus test parses each tail and re-renders it, asserting byte identity
(for the families the producer link vocabulary covers) — `method/…` tails are
listed too but marked out-of-vocabulary by the test itself.
"""
import collections
import glob
import json
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, "..", "..", "oracle", "captured_responses")
OUT = os.path.join(HERE, "..", "r5_tails")

URL_ATTR = re.compile(r'(?:href|action|dotsrc)=[\'"]([^\'"]+)[\'"]')
MAIN = re.compile(r"^/thy/(?:trace|equiv)/[^/]+/main/(.+)$")

tails = collections.Counter()
for p in sorted(glob.glob(os.path.join(CORPUS, "*.hs.json"))):
    d = json.load(open(p))
    for url, r in d.get("manifest", {}).items():
        body = r.get("body", "")
        for m in URL_ATTR.finditer(body):
            mm = MAIN.match(m.group(1))
            if mm:
                tails[mm.group(1)] += 1
        if r.get("kind") == "text" and body.startswith("/thy/"):
            mm = MAIN.match(body.strip())
            if mm:
                tails[mm.group(1)] += 1

os.makedirs(OUT, exist_ok=True)
with open(os.path.join(OUT, "tails.txt"), "w") as fh:
    for t in sorted(tails):
        fh.write("%d\t%s\n" % (tails[t], t))
print("wrote %d distinct tails (%d occurrences)" % (len(tails), sum(tails.values())))
