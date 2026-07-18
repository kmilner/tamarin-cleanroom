#!/usr/bin/env bash
# Serve a .spthy in the HS interactive UI on a given port (--port= form).
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
BIN="/home/kamilner/tamarin-rs/tamarin-prover-testing/.stack-work/install/x86_64-linux-tinfo6/ec0cb11b1bfcf8776d45e0357bbc6d6ff2077f9222735af22115429c8cdfcef1/9.6.7/bin/tamarin-prover"
case "$1" in
  start)
    d=$(mktemp -d); cp "$2" "$d/";
    nohup "$BIN" interactive "$d" --port="$3" >/tmp/wrapsrv_$3.log 2>&1 &
    echo $! > /tmp/wrapsrv_$3.pid; sleep 4; echo "up on $3 pid $(cat /tmp/wrapsrv_$3.pid)";;
  stop) kill $(cat /tmp/wrapsrv_$2.pid) 2>/dev/null; rm -f /tmp/wrapsrv_$2.pid; echo stopped;;
esac
