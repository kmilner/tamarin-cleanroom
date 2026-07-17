#!/usr/bin/env python3
"""Autoprove every lemma of a served theory, fetch its proof-state DOT, save it.
Usage: drive.py <port> <theory.spthy>   (reads lemma names from the file)"""
import sys, re, os, urllib.request
PORT = sys.argv[1]; THY = sys.argv[2]
BASE = f"http://127.0.0.1:{PORT}"
OUT = os.environ.get("OUTDIR", "/tmp/r7_dots"); os.makedirs(OUT, exist_ok=True)
LEMMAS = re.findall(r'^lemma (l_\w+):', open(THY).read(), re.M)

def get(path, timeout=600):
    with urllib.request.urlopen(BASE + path, timeout=timeout) as r:
        return r.read().decode("utf-8", "replace")

def latest_index():
    ms = re.findall(r'/thy/trace/(\d+)/', get("/"))
    return max(int(x) for x in ms) if ms else 1

for L in LEMMAS:
    i = latest_index()
    try:
        r = get(f"/thy/trace/{i}/autoprove/idfs/0/False/proof/{L}")
    except Exception as e:
        print(f"{L}\tAUTOPROVE_ERR {e}"); continue
    ni = i
    m = re.search(r'/thy/trace/(\d+)/', r)
    if m: ni = int(m.group(1))
    try:
        dot = get(f"/thy/trace/{ni}/interactive-graph-def/proof/{L}/_")
    except Exception as e:
        print(f"{L}\tGRAPH_ERR {e}"); continue
    open(os.path.join(OUT, f"{L}.dot"), "w").write(dot)
    print(f"{L}\t{len(dot)}")
