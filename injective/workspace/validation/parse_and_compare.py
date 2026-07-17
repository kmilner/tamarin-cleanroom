#!/usr/bin/env python3
"""Corpus comparator for Unit F (injective fact instances).

For every captured rules page under corpus_html/:
  * trace theories: read the oracle's "Fact Symbols with Injective Instances"
    line (ground truth), render the theory's MSR rules into the injfacts binary
    input grammar, run the crate, and compare the (name,arity) tag sets.
  * diff theories: the oracle surfaces no injective section (diff mode); recorded
    as n/a. The crate is still run on the left-projection rules for information.

Writes corpus_results.tsv. HTML text handling and rule structure are derived only
from observed oracle output (see workspace/BEHAVIOR.md); no tamarin source read.
"""
import glob, html as htmlmod, json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
HTML = os.path.join(HERE, "corpus_html")
BIN = os.path.join(HERE, "..", "injfacts-clean", "target", "release", "injfacts")

# ---- HTML -> plain text (entities + <br/> -> newline, drop other tags) -------
def to_text(h):
    h = h.replace("<br/>", "\n").replace("<br>", "\n")
    h = re.sub(r"<[^>]+>", "", h)
    return htmlmod.unescape(h)

# ---- top-level comma split (respect () depth) --------------------------------
def split_top(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch == "(":
            depth += 1; cur += ch
        elif ch == ")":
            depth -= 1; cur += ch
        elif ch == "," and depth == 0:
            if cur.strip():
                out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out

# ---- oracle injective template list -> set of (name, arity) ------------------
def parse_injective_section(text):
    # text begins after the section header; first line group is the fact list.
    m = re.search(r"Fact Symbols with Injective Instances\s*(.*?)\n\s*(?:Multiset Rewriting Rules|$)",
                  text, re.DOTALL)
    body = m.group(1).strip() if m else ""
    # body is either "None" or "F(id), G(id,=), ..."
    if not body or body == "None":
        return set()
    tags = set()
    for tmpl in split_top(body):
        mm = re.match(r"^\s*([!]?[A-Za-z0-9_]+)\s*(?:\((.*)\))?\s*$", tmpl, re.DOTALL)
        if not mm:
            continue
        name = mm.group(1)
        argstr = mm.group(2)
        arity = 0 if argstr is None or argstr.strip() == "" else len(split_top(argstr))
        tags.add((name, arity))
    return tags

# ---- render one fact string ("[!]Name( a, b )") into a binary token line -----
def fact_to_tokens(fact):
    fact = fact.strip()
    persistent = fact.startswith("!")
    core = fact[1:].strip() if persistent else fact
    mm = re.match(r"^([A-Za-z0-9_]+)\s*(?:\((.*)\))?\s*$", core, re.DOTALL)
    if not mm:
        return None
    name = mm.group(1)
    argstr = mm.group(2)
    args = [] if argstr is None else split_top(argstr)
    toks = [("!" if persistent else "") + name]
    for a in args:
        toks.append(re.sub(r"\s+", "", a))  # collapse to a single whitespace-free token
    return toks

# ---- extract the main rewrite of every rule chunk in a rules block -----------
# A rule header is `rule (modulo X) NAME[ATTRS]:` where the optional [ATTRS]
# block (sapic rules: color/process/issapicrule, spanning several lines and
# containing colons) sits between the name and the terminating colon.
HEADER = re.compile(r"rule\s*\(modulo\s+\w+\)\s*[^\[\n:]*(?:\[[^\]]*\])?\s*:")
MAIN = re.compile(r"\[(.*?)\]\s*(?:--\[.*?\]->|-->)\s*\[(.*?)\]", re.DOTALL)

def rules_to_binput(rules_text):
    # remove /* ... */ comments
    rules_text = re.sub(r"/\*.*?\*/", " ", rules_text, flags=re.DOTALL)
    headers = list(HEADER.finditer(rules_text))
    lines = []
    for i, h in enumerate(headers):
        start = h.end()
        end = headers[i + 1].start() if i + 1 < len(headers) else len(rules_text)
        chunk = rules_text[start:end]
        m = MAIN.search(chunk)
        if not m:
            continue
        lines.append("RULE")
        for kind, seg in (("P", m.group(1)), ("C", m.group(2))):
            for fact in split_top(seg):
                toks = fact_to_tokens(fact)
                if toks:
                    lines.append(kind + " " + " ".join(toks))
    return "\n".join(lines) + "\n"

def run_crate(binput):
    r = subprocess.run([BIN], input=binput, capture_output=True, text=True)
    tags = set()
    for ln in r.stdout.splitlines():
        ln = ln.strip()
        if not ln:
            continue
        name, ar = ln.rsplit("/", 1)
        tags.add((name, int(ar)))
    return tags

def fmt(tags):
    return ", ".join(f"{n}/{a}" for n, a in sorted(tags)) if tags else "None"

# --- diff: pull left-projection rules (between 'left' and 'right' markers) -----
def diff_left_rules_text(rules_text):
    rules_text = re.sub(r"/\*.*?\*/", " ", rules_text, flags=re.DOTALL)
    out = []
    # keep every 'rule ...' whose header is immediately preceded by a 'left'
    # marker, plus intruder rules that have no left/right pairing.
    # Simplify: take the whole block but drop parent+right by slicing left..right.
    for m in re.finditer(r"\bleft\b(.*?)\bright\b", rules_text, re.DOTALL):
        out.append(m.group(1))
    # also include rules that are not part of a diff triple (intruder rules):
    return "\n".join(out)

def main():
    rows = []
    files = sorted(glob.glob(os.path.join(HTML, "*.json")))
    for path in files:
        base = os.path.basename(path)[:-5]  # strip .json
        m = re.match(r"^(.*)\.(trace|equiv)(\d+)$", base)
        if not m:
            continue
        slug, route, idx = m.group(1), m.group(2), m.group(3)
        theory = slug.replace("__", "/") + ".spthy"
        try:
            h = json.load(open(path))["html"]
        except Exception as e:
            rows.append((theory, route, "ERR", "ERR", "load-error", str(e)[:40]))
            continue
        text = to_text(h)
        if route == "trace":
            oracle = parse_injective_section(text)
            mm = re.search(r"Multiset Rewriting Rules(.*)$", text, re.DOTALL)
            rules_text = mm.group(1) if mm else ""
            binput = rules_to_binput(rules_text)
            clean = run_crate(binput)
            agree = "MATCH" if oracle == clean else "MISMATCH"
            note = ""
            if oracle != clean:
                note = "oracle-only=%s clean-only=%s" % (fmt(oracle - clean), fmt(clean - oracle))
            rows.append((theory, route, fmt(oracle), fmt(clean), agree, note))
        else:  # diff
            left = diff_left_rules_text(text)
            binput = rules_to_binput(left) if left.strip() else "\n"
            clean = run_crate(binput)
            rows.append((theory, "diff", "(no injective section in diff mode)",
                         fmt(clean) + " (informational, left-proj)", "n/a-diff", ""))
    # also record theories that failed to scrape (status FAIL)
    scraped = {re.match(r"^(.*)\.(trace|equiv)\d+$", os.path.basename(p)[:-5]).group(1)
               for p in files if re.match(r"^(.*)\.(trace|equiv)\d+$", os.path.basename(p)[:-5])}
    for st in sorted(glob.glob(os.path.join(HTML, "*.status"))):
        slug = os.path.basename(st)[:-7]
        content = open(st).read().strip()
        if content.startswith("FAIL") and slug not in scraped:
            theory = slug.replace("__", "/") + ".spthy"
            rows.append((theory, "?", "ERR", "ERR", "scrape-fail", content))

    out = os.path.join(HERE, "corpus_results.tsv")
    with open(out, "w") as f:
        f.write("theory\tmode\toracle_injective\tclean_injective\tagreement\tnote\n")
        for r in sorted(rows):
            f.write("\t".join(str(x) for x in r) + "\n")
    # console summary
    from collections import Counter
    c = Counter(r[4] for r in rows)
    print("rows:", len(rows), dict(c))
    for r in rows:
        if r[4] not in ("MATCH", "n/a-diff"):
            print("  !!", r[0], r[4], r[5])

if __name__ == "__main__":
    main()
