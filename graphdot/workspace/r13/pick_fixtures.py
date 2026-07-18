#!/usr/bin/env python3
"""Pick ~24 diverse captures covering the 3 new styles, info-ports, clusters,
legends; prefer small files. Print chosen names + a coverage report."""
import os, re, collections

CORPUS = "/home/kamilner/tamarin-cleanroom/graphdot/oracle/dot_corpus"
edge_re = re.compile(r'^(n\d+)(?::(n\d+))? -> (n\d+)(?::(n\d+))?(\[[^\]]*\])?;$')
rec_re = re.compile(r'^n\d+\[shape="record"')

def features(path):
    with open(path, encoding='utf-8') as f:
        txt = f.read()
    feats = set()
    if 'color="purple",style="dashed"' in txt: feats.add('purple')
    if 'style="dotted",color="green"' in txt: feats.add('green')
    if 'color="darkorange3",style="dashed"' in txt: feats.add('darkorange3')
    if 'packmode="cluster"' in txt: feats.add('cluster')
    if 'shape="plain"' in txt: feats.add('legend')
    if 'color="blue3",style="dashed"' in txt: feats.add('blue3')
    if 'color="black",style="dashed"' in txt: feats.add('black')
    # info-port edge present? any edge nX:nY where nY is a middle-col port
    if re.search(r'n\d+:n\d+ -> ', txt): feats.add('srcport')
    if re.search(r'-> n\d+:n\d+', txt): feats.add('dstport')
    return feats, len(txt)

files = [f for f in os.listdir(CORPUS) if f.endswith('.dot')]
# gather features + size
info = {}
for f in sorted(files):
    p = os.path.join(CORPUS, f)
    feats, sz = features(p)
    info[f] = (feats, sz)

witnesses = ['01c5db0a7030e664.dot', '00082e1d6a47b5af.dot']
chosen = []
want = collections.Counter()

def add(f):
    if f not in chosen:
        chosen.append(f)
        for x in info[f][0]:
            want[x]+=1

for w in witnesses:
    add(w)

# For each rare feature, pick the smallest files carrying it.
for feat in ['purple','green','darkorange3']:
    cands = sorted([f for f in files if feat in info[f][0]], key=lambda f: info[f][1])
    for f in cands[:4]:
        add(f)

# add some small cluster + legend + blue3/black graphs
for feat in ['cluster','legend','blue3','black']:
    cands = sorted([f for f in files if feat in info[f][0]], key=lambda f: info[f][1])
    for f in cands[:2]:
        add(f)

# a few small plain graphs for diversity
small = sorted(files, key=lambda f: info[f][1])
for f in small[:4]:
    add(f)

print("CHOSEN", len(chosen), "files, total bytes",
      sum(info[f][1] for f in chosen))
allfeat = collections.Counter()
for f in chosen:
    allfeat.update(info[f][0])
print("coverage:", dict(allfeat))
for f in chosen:
    feats, sz = info[f]
    print(f"{sz:7d}  {f}  {sorted(feats)}")
