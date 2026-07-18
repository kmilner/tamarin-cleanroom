#!/usr/bin/env python3
# Autoprove each lemma; fetch the proof graph DOT twice: default and with
# ?unabbreviate= (internal ground truth). Usage: drive2.py <port> <lemma...>
import sys, re, urllib.request, os
PORT=sys.argv[1]; LEMMAS=sys.argv[2:]
BASE=f"http://127.0.0.1:{PORT}"
OUT=os.environ.get("OUTDIR","/tmp/probe_dots"); os.makedirs(OUT,exist_ok=True)
def get(path, timeout=300):
    with urllib.request.urlopen(BASE+path, timeout=timeout) as r:
        return r.read().decode("utf-8","replace")
def latest_index():
    idx=get("/")
    ms=re.findall(r'/thy/trace/(\d+)/', idx)
    return max(int(x) for x in ms) if ms else 1
for L in LEMMAS:
    i=latest_index()
    try:
        r=get(f"/thy/trace/{i}/autoprove/idfs/0/False/proof/{L}")
    except Exception as e:
        print(f"{L}\tAUTOPROVE_ERR {e}"); continue
    ni=i
    m=re.search(r'/thy/trace/(\d+)/', r)
    if m: ni=int(m.group(1))
    try:
        dot=get(f"/thy/trace/{ni}/interactive-graph-def/proof/{L}/_")
        dotu=get(f"/thy/trace/{ni}/interactive-graph-def/proof/{L}/_?unabbreviate=")
    except Exception as e:
        print(f"{L}\tGRAPH_ERR {e}"); continue
    p=os.path.join(OUT,f"{L}.dot"); open(p,"w").write(dot)
    pu=os.path.join(OUT,f"{L}.unabbr.dot"); open(pu,"w").write(dotu)
    print(f"{L}\t{p}\t{len(dot)}\t{len(dotu)}")
