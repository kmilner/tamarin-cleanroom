#!/usr/bin/env python3
"""Harvest the href/action/dotsrc URL vocabulary out of the captured corpus.

Reads ONLY oracle/captured_responses/*.hs.json (captured program OUTPUT).
Every URL that appears inside a response BODY (href="…", action="…",
dotsrc="…", plus the bare-URL text bodies of next/prev) is collected,
bucketed by its /thy/<kind>/<idx>/<handler>/ skeleton, and inventoried:

  families    per-family counts + example URLs
  segments    distinct segment shapes per family position (encoding survey)
  encoded     every URL containing a % escape or other non-alnum segment bytes
  chars       the exact byte inventory of <lemma>/<case>/<pos> style segments
"""
import collections
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, "..", "..", "oracle", "captured_responses")

URL_ATTR = re.compile(r'(?:href|action|dotsrc)=[\'"]([^\'"]+)[\'"]')
IDX = re.compile(r"^/thy/(trace|equiv)/([^/]+)/")


def urls():
    for p in sorted(glob.glob(os.path.join(CORPUS, "*.hs.json"))):
        d = json.load(open(p))
        for url, r in d.get("manifest", {}).items():
            body = r.get("body", "")
            kind = r.get("kind")
            for m in URL_ATTR.finditer(body):
                yield os.path.basename(p), url, m.group(1)
            if kind == "text" and body.startswith("/thy/"):
                yield os.path.basename(p), url, body.strip()


def main(cmd):
    fam = collections.Counter()
    fam_ex = {}
    seg_shapes = collections.defaultdict(collections.Counter)
    encoded = collections.Counter()
    seg_chars = collections.Counter()
    n = 0
    for src, page, u in urls():
        n += 1
        m = IDX.match(u)
        if not m:
            fam["NON-THY:" + u.split("/")[1] if u.startswith("/") and len(u) > 1 else u] += 1
            fam_ex.setdefault("NON-THY", u)
            continue
        tail = u[m.end():]
        segs = tail.split("/")
        handler = segs[0]
        if handler == "main" and len(segs) > 1:
            fam_key = "main/" + segs[1]
            args = segs[2:]
        else:
            fam_key = handler
            args = segs[1:]
        fam[fam_key] += 1
        fam_ex.setdefault(fam_key, u)
        for i, s in enumerate(args):
            if re.fullmatch(r"[0-9]+", s):
                shape = "<NUM>"
            elif re.fullmatch(r"[A-Za-z0-9_.-]+", s):
                shape = "<IDENTISH>"
            else:
                shape = "<OTHER:%s>" % s
            seg_shapes[fam_key][(i, shape)] += 1
            for ch in s:
                if not (ch.isalnum()):
                    seg_chars[ch] += 1
        if "%" in u:
            encoded[u] += 1

    if cmd == "families":
        for f, c in sorted(fam.items(), key=lambda x: -x[1]):
            print("%7d  %-28s  ex: %s" % (c, f, fam_ex.get(f, "")))
    elif cmd == "segments":
        for f in sorted(seg_shapes):
            print("== %s" % f)
            for (i, shape), c in sorted(seg_shapes[f].items()):
                print("   arg[%d] %-18s x%d" % (i, shape, c))
    elif cmd == "encoded":
        for u, c in sorted(encoded.items()):
            print("%6d  %s" % (c, u))
        print("-- %d distinct URLs with %% escapes (of %d total)" % (len(encoded), n))
    elif cmd == "chars":
        for ch, c in sorted(seg_chars.items()):
            print("%r x%d" % (ch, c))
    print("-- scanned %d urls" % n, file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "families")
