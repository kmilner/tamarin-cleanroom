#!/usr/bin/env bash
# gprobe5.sh <tag> <trace: all|exists> <formula> [extra-header-lines]
# Theory with rule R1 actions A(~x), B(~x,~x), C(~x); prints wf block.
tag="$1"; trace="$2"; formula="$3"; extra="${4:-}"
d=/home/kamilner/tamarin-cleanroom/wellformedness/workspace/scratchpad/probes5
q="all-traces"; [ "$trace" = "exists" ] && q="exists-trace"
cat > "$d/$tag.spthy" <<THY
theory T$tag
begin
$extra
functions: h/1
rule R1:
   [ Fr(~x) ]
  --[ A(~x), B(~x,~x), C(~x) ]->
   [ Out(~x) ]
lemma L:
  $q
  "$formula"
end
THY
/home/kamilner/tamarin-cleanroom/wellformedness/workspace/scratchpad/wf.sh "$d/$tag.spthy"
