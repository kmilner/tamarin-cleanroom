#!/usr/bin/env python3
"""Slice fragment families out of the captured response corpus.

The captured corpus (`oracle/captured_responses/*.hs.json`, the sanctioned
channel PROTOCOL permits reading) holds, per theory, a crawl manifest:

    { base, capped, lemmas, log, manifest: { "<url>": {kind,status,body} } }

`<url>` carries the literal token `#` where the server's version index goes;
`kind` is one of html | text | json | dot; a `json` body is itself the JSON
envelope string `{"html":..,"title":..}` (or `{"redirect":..}`/`{"alert":..}`).

This helper lets the sealed room materialize the *exact response bytes* for one
fragment family without re-crawling — the byte targets a producer must
reproduce. It reads ONLY the captured corpus (no corpus `.spthy`, no source
tree); it labels each manifest by the theory name it advertises plus a short
content hash so name-collisions (Joux, Scott, …) stay distinct.

Usage:
  extract_fragments.py list                       # manifests: label / name / kinds
  extract_fragments.py families                    # URL families + counts + kinds
  extract_fragments.py extract <family[,family2,...]> <outdir> [--only SUBSTR[,SUBSTR2,...]]
        # <family> is a URL skeleton, idx-agnostic; pass several comma-separated
        # to slice them all in ONE corpus pass, e.g.:
        #   overview   main/message   main/rules   main/tactic   main/help
        #   main/lemma   source   next   main/cases   main/proof
        # writes one file per (manifest, matching-url): <label>__<slug>.<ext>
        # json envelopes also emit a sibling .title file (the pane title).
  --only SUBSTR  restrict to manifests whose label CONTAINS any listed SUBSTR
                 (comma-separated). Omit to take every manifest.

All output files are captured program OUTPUT (legitimate byte targets); the
`.meta.tsv` index records label -> theory-name -> capture-hash for traceability.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.environ.get("CAPTURES", os.path.join(HERE, "captured_responses"))

_IDX = re.compile(r"/thy/(trace|equiv)/[^/]+/")


def label_of(path, manifest):
    """theory-name + 8-char capture hash (collision-proof, capture-only)."""
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
    name = re.sub(r"[^A-Za-z0-9_.-]", "_", name or "UNKNOWN")
    return "%s__%s" % (name, h), name, h


def family_of(url):
    """idx-agnostic URL skeleton down to the handler section."""
    u = _IDX.sub("/thy/trace/#/", url)
    m = re.match(r"^/thy/trace/#/(main/[a-zA-Z]+|[a-zA-Z-]+)", u)
    if m:
        return m.group(1)
    return u.strip("/") or "root"


def slug(url):
    return re.sub(r"[^A-Za-z0-9]+", "_", _IDX.sub("/thy/trace/#/", url)).strip("_")


def manifests():
    for p in sorted(__import__("glob").glob(os.path.join(CORPUS, "*.hs.json"))):
        try:
            yield p, json.load(open(p))
        except Exception as e:  # noqa
            print("skip %s: %s" % (p, e), file=sys.stderr)


def cmd_list():
    for p, d in manifests():
        m = d.get("manifest", {})
        lbl, name, h = label_of(p, m)
        kinds = sorted({r.get("kind") for r in m.values()})
        print("%-40s  %-28s  urls=%-5d  kinds=%s" % (lbl, name, len(m), ",".join(kinds)))


def cmd_families():
    import collections
    fam = collections.Counter()
    famk = collections.defaultdict(set)
    for _, d in manifests():
        for url, r in d.get("manifest", {}).items():
            f = family_of(url)
            fam[f] += 1
            famk[f].add(r.get("kind"))
    for f, c in sorted(fam.items(), key=lambda x: -x[1]):
        print("%6d  %-24s  %s" % (c, f, ",".join(sorted(famk[f]))))


def _write(outdir, base, body, title=None):
    os.makedirs(outdir, exist_ok=True)
    ext = "html" if ("<" in body or title is not None) else "txt"
    with open(os.path.join(outdir, base + "." + ext), "w") as fh:
        fh.write(body)
    if title is not None:
        with open(os.path.join(outdir, base + ".title"), "w") as fh:
            fh.write(title)


def cmd_extract(family, outdir, only=None):
    fams = set(f.strip() for f in family.split(",") if f.strip())
    subs = [s.strip() for s in only.split(",")] if only else None
    n = 0
    meta = []
    for p, d in manifests():
        m = d.get("manifest", {})
        lbl, name, h = label_of(p, m)
        if subs and not any(s in lbl for s in subs):
            continue
        for url, r in m.items():
            if family_of(url) not in fams:
                continue
            body = r.get("body", "")
            title = None
            if r.get("kind") == "json":
                try:
                    env = json.loads(body)
                except Exception:
                    env = None
                if isinstance(env, dict) and "html" in env:
                    title = env.get("title", "")
                    body = env["html"]
                # {redirect}/{alert} envelopes are left as raw JSON (no html key).
            base = "%s__%s" % (lbl, slug(url))
            _write(outdir, base, body, title)
            meta.append((base, name, h, url, r.get("kind"), r.get("status")))
            n += 1
    with open(os.path.join(outdir, ".meta.tsv"), "w") as fh:
        fh.write("target\ttheory\thash\turl\tkind\tstatus\n")
        for row in meta:
            fh.write("\t".join(str(x) for x in row) + "\n")
    print("wrote %d '%s' fragments to %s" % (n, family, outdir))


def main(argv):
    if not argv or argv[0] == "list":
        return cmd_list()
    if argv[0] == "families":
        return cmd_families()
    if argv[0] == "extract":
        family = argv[1]
        outdir = argv[2]
        only = None
        if "--only" in argv:
            only = argv[argv.index("--only") + 1]
        return cmd_extract(family, outdir, only)
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]) or 0)
