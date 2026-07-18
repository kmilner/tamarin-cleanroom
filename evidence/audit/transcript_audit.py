#!/usr/bin/env python3
"""Clean-room transcript audit: verifies no forbidden file access and no web
access in agent tool-call transcripts. Allowed: paths under tamarin-cleanroom,
the captured-output cache (.web_hs_cache), and executing the sanctioned HS
oracle binary (tamarin-prover-testing/.stack-work). Everything else under
/home/kamilner/tamarin-rs/ is a violation for CLEAN-ROOM agents (auditors and
integrators are exempt by role — they are both-sides by design)."""
import json, re, sys, glob
# .spthy example theories are sanctioned INPUT material (PROTOCOL.md), whether
# reached via a cluster's oracle/examples symlink or the raw submodule path —
# rule refined 2026-07-18 after a false positive on raw-path example reads.
ALLOWED = ("tamarin-cleanroom", ".web_hs_cache", "tamarin-prover-testing/.stack-work",
           "wf_oracle", "hs_server", "tamarin-prover/examples")
# Hard override: direct use of a forbidden path is a violation even when an
# allowed substring co-occurs in the same command (added 2026-07-18; stricter
# than the original any()-whitelist).
FORBIDDEN = re.compile(r"/home/kamilner/tamarin-rs/(?:[^\"'\s]*\.hs\b|crates/)")
def audit(path):
    viol, web, tools = [], 0, 0
    for line in open(path, errors="replace"):
        try: r = json.loads(line)
        except: continue
        for blk in (r.get("message", {}).get("content") or []):
            if not isinstance(blk, dict) or blk.get("type") != "tool_use": continue
            tools += 1
            s = json.dumps(blk.get("input", {}))
            if blk.get("name") in ("WebFetch", "WebSearch"): web += 1
            hit = ("/home/kamilner/tamarin-rs/" in s and not any(a in s for a in ALLOWED))
            if hit or FORBIDDEN.search(s):
                viol.append((blk.get("name"), s[:160]))
    return tools, web, viol
if __name__ == "__main__":
    for p in sys.argv[1:] or sorted(glob.glob("**/agent-*.jsonl", recursive=True)):
        t, w, v = audit(p)
        print(f"{p}: tool_calls={t} web={w} VIOLATIONS={len(v)}")
        for name, s in v: print(f"   !! {name} {s}")
