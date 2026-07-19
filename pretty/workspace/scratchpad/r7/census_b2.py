#!/usr/bin/env python3
"""R7 verification: reproduce the B2 boundary census over all message-pane rule
bodies. one-line max escaped width should be 67, wrapped min should be 69."""
import glob, json, os, re, sys

RAW = "/home/kamilner/tamarin-cleanroom/pretty/workspace/scratchpad/r6/raw"

def unescape(s):
    return (s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", '"')
             .replace("&#39;", "'").replace("&amp;", "&"))

def strip_spans(s):
    return re.sub(r"</?span[^>]*>", "", s)

def esc_w(s):
    w = 0
    for c in s:
        if c in "<>":
            w += 4
        elif c == "&":
            w += 5
        elif c == '"':
            w += 6
        elif c == "'":
            w += 5
        else:
            w += 1
    return w

def unpostprocess(html):
    # split into logical lines, converting &nbsp; runs to leading spaces
    assert html.endswith("<br/>\n")
    parts = html.split("<br/>\n")[:-1]
    out = []
    for line in parts:
        n = 0
        while line.startswith("&nbsp;"):
            line = line[6:]
            n += 1
        out.append(" " * n + line)
    return out

def parse_blocks(lines):
    i = 0
    blocks = []
    while i < len(lines):
        m = re.match(r"<h2>(.*)</h2>$", lines[i])
        assert m, lines[i]
        head = m.group(1)
        i += 1
        first = lines[i]
        assert first.startswith('<p class="monospace rules">'), first
        cur = first[len('<p class="monospace rules">'):]
        body = []
        while True:
            if cur.endswith("</p>"):
                stripped = cur[:-4]
                if not (stripped == "" and not body):
                    body.append(stripped)
                i += 1
                break
            body.append(cur)
            i += 1
            cur = lines[i]
        blocks.append((head, body))
    return blocks

def plain(body):
    return [unescape(strip_spans(l)) for l in body]

one_line = []   # (esc_width, file, text)
wrapped = []    # (esc_width_reconstructed, file, exception?)

for raw_path in sorted(glob.glob(os.path.join(RAW, "*__message.raw"))):
    raw = open(raw_path).read()
    env = json.loads(raw)
    if env["title"] != "Message theory":
        continue
    fname = os.path.basename(raw_path)
    lines = unpostprocess(env["html"])
    blocks = parse_blocks(lines)
    for head, body in blocks:
        if head not in ("Construction Rules", "Deconstruction Rules"):
            continue
        pl = plain(body)
        # split into rule blocks at col-0 "rule "
        idxs = [k for k, l in enumerate(pl) if l.startswith("rule ")]
        for j, a in enumerate(idxs):
            end = idxs[j + 1] if j + 1 < len(idxs) else len(pl)
            blk = pl[a:end]
            # drop trailing blank lines
            while blk and blk[-1].strip() == "":
                blk.pop()
            bodyrows = blk[1:]  # after header
            # trim leading blank
            while bodyrows and bodyrows[0].strip() == "":
                bodyrows.pop(0)
            if not bodyrows:
                continue
            if len(bodyrows) == 1:
                content = bodyrows[0].lstrip(" ")
                one_line.append((esc_w(content), fname, content))
            else:
                # reconstruct one-line: strip each row, join with single spaces
                stripped = [r.strip() for r in bodyrows if r.strip() != ""]
                recon = " ".join(stripped)
                # exception if any bracket group internally wrapped: heuristic =
                # a row that does not itself look like a complete [ ... ] or
                # --[ ... ]-> or [ ... ] group (unbalanced bracket count)
                exc = False
                for r in stripped:
                    if r.count("[") != r.count("]"):
                        exc = True
                wrapped.append((esc_w(recon), fname, exc, recon))

ol_w = [w for w, _, _ in one_line]
wr_clean = [(w, f, r) for (w, f, e, r) in wrapped if not e]
wr_exc = [(w, f, r) for (w, f, e, r) in wrapped if e]
wr_w = [w for w, _, _ in wr_clean]

print(f"one-line bodies: {len(one_line)}  max escaped width = {max(ol_w)} (glyph-max would be {max(len(t) for _,_,t in one_line)})")
print(f"wrapped bodies (clean): {len(wr_clean)}  min reconstructed escaped width = {min(wr_w)}")
print(f"wrapped bodies (inner-wrapped exceptions, excluded): {len(wr_exc)}")

# show the boundary witnesses
ol_sorted = sorted(one_line, reverse=True)
print("\nTop 3 widest one-line bodies:")
for w, f, t in ol_sorted[:3]:
    print(f"  esc={w} glyph={len(t)}  {f}\n     {t}")
wr_sorted = sorted(wr_clean)
print("\nNarrowest 3 wrapped (clean) bodies:")
for w, f, r in wr_sorted[:3]:
    print(f"  esc={w}  {f}\n     {r}")

# classification check: does "escaped <= 67 => one line" hold with zero clean exceptions?
misclass = [(w,f,t) for w,f,t in one_line if w > 67]
print(f"\none-line bodies with escaped width > 67 (law violations): {len(misclass)}")
for w,f,t in misclass:
    print(f"  esc={w} {f}: {t}")
misclass2 = [(w,f,r) for w,f,r in wr_clean if w <= 67]
print(f"wrapped clean bodies with escaped width <= 67 (law violations): {len(misclass2)}")
for w,f,r in misclass2:
    print(f"  esc={w} {f}: {r}")
