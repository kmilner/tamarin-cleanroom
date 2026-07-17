#!/usr/bin/env bash
# gprobe.sh <tag> <trace: all|exists> <formula> [builtins]
# Builds a theory with rule R1 (A,B,C,P actions) and one lemma, prints the
# wellformedness block. Fact facts available in rule: A,B,C over ~x.
tag="$1"; trace="$2"; formula="$3"; builtins="${4:-}"
d=/home/kamilner/tamarin-cleanroom/wellformedness/workspace/scratchpad/probes4
q="all-traces"; [ "$trace" = "exists" ] && q="exists-trace"
cat > "$d/$tag.spthy" <<EOF
theory T$tag
begin
$builtins
predicates: Pr(x) <=> x = x
rule R1:
   [ Fr(~x) ]
  --[ A(~x), B(~x), C(~x) ]->
   [ Out(~x) ]
lemma L:
  $q
  "$formula"
end
EOF
/home/kamilner/tamarin-cleanroom/wellformedness/workspace/scratchpad/wf.sh "$d/$tag.spthy"
