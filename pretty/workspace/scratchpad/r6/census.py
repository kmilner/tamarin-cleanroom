#!/usr/bin/env python3
"""Census the web-mode rendering from extracted main/message + main/rules bodies.

Input: the .html files written by extract_fragments.py (decoded env["html"]
fragments). We invert the producers skin postprocess to recover the BODY lines
(the opaque content this round must produce), then census spans/escapes/width.
"""
import glob, os, re, sys, collections

PANES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "panes")
P_OPEN = '<p class="monospace rules">'

def unpostprocess(html):
    assert html.endswith("<br/>\n"), repr(html[-20:])
    lines = html.split("<br/>\n")
    assert lines[-1] == ""
    lines = lines[:-1]
    out = []
    for line in lines:
        n = 0
        while line.startswith("&nbsp;"):
            line = line[6:]
            n += 1
        out.append(" " * n + line)
    return out

def parse_blocks(lines):
    """Return list of (heading, [body-lines]) and leading_blank bool."""
    i = 0
    leading_blank = bool(lines) and lines[0] == ""
    if leading_blank:
        i = 1
    blocks = []
    while i < len(lines):
        m = re.match(r"^<h2>(.*)</h2>$", lines[i])
        assert m, "expected heading, got %r (idx %d)" % (lines[i], i)
        heading = m.group(1)
        i += 1
        assert lines[i].startswith(P_OPEN), "expected p open, got %r" % lines[i]
        first = lines[i][len(P_OPEN):]
        body = []
        cur = first
        while True:
            if cur.endswith("</p>"):
                stripped = cur[:-4]
                if not (stripped == "" and body == []):
                    body.append(stripped)
                i += 1
                break
            body.append(cur)
            i += 1
            cur = lines[i]
        blocks.append((heading, body))
    return leading_blank, blocks

def strip_spans(s):
    return re.sub(r'</?span[^>]*>', '', s)

def unescape(s):
    return (s.replace("&lt;","<").replace("&gt;",">").replace("&quot;",'"')
             .replace("&#39;","'").replace("&amp;","&"))

def main():
    what = sys.argv[1] if len(sys.argv) > 1 else "census"
    msg_files = sorted(glob.glob(os.path.join(PANES, "*main_message.html")))
    rules_files = sorted(glob.glob(os.path.join(PANES, "*main_rules.html")))

    if what == "census":
        span_classes = collections.Counter()
        # token census: for each span, record class + inner content
        span_inner = collections.defaultdict(collections.Counter)
        entities = collections.Counter()
        headings = collections.Counter()
        for f in msg_files + rules_files:
            html = open(f).read()
            lines = unpostprocess(html)
            lb, blocks = parse_blocks(lines)
            for h, body in blocks:
                headings[h] += 1
                for bl in body:
                    for m in re.finditer(r'<span class="([^"]+)">(.*?)</span>', bl):
                        span_classes[m.group(1)] += 1
                        span_inner[m.group(1)][m.group(2)] += 1
                    for m in re.finditer(r'&[a-z#0-9]+;', bl):
                        entities[m.group(0)] += 1
        print("=== SPAN CLASSES ===")
        for c, n in span_classes.most_common():
            print("  %-16s %d" % (c, n))
        print("\n=== SPAN INNER CONTENT by class (distinct) ===")
        for c in span_inner:
            print("class=%s: %d distinct inners" % (c, len(span_inner[c])))
            for inner, n in span_inner[c].most_common(60):
                print("    %6d  %r" % (n, inner))
        print("\n=== ENTITIES ===")
        for e, n in entities.most_common():
            print("  %-10s %d" % (e, n))
        print("\n=== HEADINGS ===")
        for h, n in headings.most_common():
            print("  %-45s %d" % (h, n))

    elif what == "width":
        # For each body line, strip spans + unescape -> visible plain text.
        # Report max visible length and the distribution near the top.
        maxlen = 0
        maxline = None
        lens = collections.Counter()
        long_lines = []
        for f in msg_files + rules_files:
            html = open(f).read()
            lines = unpostprocess(html)
            lb, blocks = parse_blocks(lines)
            for h, body in blocks:
                for bl in body:
                    plain = unescape(strip_spans(bl))
                    L = len(plain)
                    lens[L] += 1
                    if L > maxlen:
                        maxlen = L; maxline = (os.path.basename(f), h, plain)
                    if L >= 95:
                        long_lines.append((L, os.path.basename(f)[:30], plain))
        print("max visible plain length:", maxlen)
        print("max line:", maxline)
        print("\nlength distribution >=90:")
        for L in sorted(lens):
            if L >= 90:
                print("  len %3d : %d lines" % (L, lens[L]))
        print("\nlongest lines (>=95):")
        for L, fn, pl in sorted(long_lines, reverse=True)[:40]:
            print("  %3d %-30s |%s|" % (L, fn, pl))

if __name__ == "__main__":
    main()
