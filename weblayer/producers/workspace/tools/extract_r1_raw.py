#!/usr/bin/env python3
"""Materialize the R1 fragment families' RAW response-body bytes.

Walks the sanctioned captured corpus (oracle/captured_responses/*.hs.json,
captured program OUTPUT) and writes, per manifest, the four center-fragment
responses' raw JSON-envelope bodies — the exact bytes the server sent — into
workspace/r1_raw/ as <theory>__<hash8>__<family>.raw. These are the byte
targets tests/corpus_sweep.rs replays the producer against (SPEC acceptance
ladder rung 2). Unlike oracle/extract_fragments.py this does NOT decode the
envelope: the sweep asserts envelope bytes too.

Reads ONLY the captured corpus. Labeling mirrors extract_fragments.py (theory
name advertised by the capture itself + 8-char capture hash).
"""
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, "..", "..", "oracle", "captured_responses")
OUT = os.path.join(HERE, "..", "r1_raw")

FAMILIES = ("main/message", "main/rules", "main/tactic", "main/help")
_IDX = re.compile(r"/thy/(trace|equiv)/[^/]+/")


def label_of(path, manifest):
    h = os.path.basename(path).split(".")[0][:8]
    name = None
    for url, r in manifest.items():
        if r.get("kind") == "html" and "/overview/" in url:
            m = re.search(r"<title>Theory:\s*([^<]*)</title>", r.get("body", ""))
            if m:
                name = m.group(1).strip()
                break
    if name is None:
        for url, r in manifest.items():
            if r.get("kind") == "text" and url.endswith("/source"):
                m = re.match(r"theory\s+(\S+)", r.get("body", ""))
                if m:
                    name = m.group(1)
                    break
    return "%s__%s" % (re.sub(r"[^A-Za-z0-9_.-]", "_", name or "UNKNOWN"), h)


def main():
    os.makedirs(OUT, exist_ok=True)
    n = 0
    rows = []
    for p in sorted(glob.glob(os.path.join(CORPUS, "*.hs.json"))):
        d = json.load(open(p))
        m = d.get("manifest", {})
        lbl = label_of(p, m)
        for url, r in m.items():
            u = _IDX.sub("/thy/trace/#/", url)
            fam = u[len("/thy/trace/#/"):] if u.startswith("/thy/trace/#/") else None
            if fam not in FAMILIES or r.get("kind") != "json":
                continue
            base = "%s__%s" % (lbl, fam.replace("/", "_"))
            with open(os.path.join(OUT, base + ".raw"), "w") as fh:
                fh.write(r["body"])
            rows.append((base, url, str(r.get("status"))))
            n += 1
    with open(os.path.join(OUT, ".meta.tsv"), "w") as fh:
        fh.write("target\turl\tstatus\n")
        for row in sorted(rows):
            fh.write("\t".join(row) + "\n")
    print("wrote %d raw fragments to %s" % (n, OUT))


if __name__ == "__main__":
    sys.exit(main())
