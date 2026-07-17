#!/usr/bin/env python3
"""Sanity audit: how many rules did the parser extract per trace theory, and how
many injective candidates (facts in both a premise and conclusion of one rule)?
A None/None MATCH is only meaningful if rules actually parsed. Flags theories
where 0 rules were extracted (parse gap) or where candidates existed but the
clean set is empty (worth eyeballing)."""
import glob, json, os, re
import parse_and_compare as P

HTML = P.HTML
for path in sorted(glob.glob(os.path.join(HTML, "*.trace*.json"))):
    base = os.path.basename(path)[:-5]
    m = re.match(r"^(.*)\.trace(\d+)$", base)
    if not m:
        continue
    theory = m.group(1)
    try:
        h = json.load(open(path))["html"]
    except Exception as e:
        print(f"{theory}\tLOAD-ERR")
        continue
    text = P.to_text(h)
    mm = re.search(r"Multiset Rewriting Rules(.*)$", text, re.DOTALL)
    rules_text = mm.group(1) if mm else ""
    binput = P.rules_to_binput(rules_text)
    nrules = binput.count("RULE")
    # count candidate tags (linear fact in both prem and conc of one rule)
    oracle = P.parse_injective_section(text)
    clean = P.run_crate(binput)
    flag = ""
    if nrules == 0:
        flag = "  <== 0 RULES PARSED"
    print(f"{nrules:4d} rules  oracle={P.fmt(oracle):22s} clean={P.fmt(clean):22s} {theory}{flag}")
