#!/usr/bin/env bash
# Black-box oracle: serve a .spthy in the HS interactive web UI.
# Usage: hs_server.sh start <file.spthy> <port>   (then curl http://127.0.0.1:<port>/...)
#        hs_server.sh stop <port>
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
case "$1" in
  start) d=$(mktemp -d); cp "$2" "$d/"; nohup "/home/kamilner/tamarin-rs/tamarin-prover-testing/.stack-work/install/x86_64-linux-tinfo6/ec0cb11b1bfcf8776d45e0357bbc6d6ff2077f9222735af22115429c8cdfcef1/9.6.7/bin/tamarin-prover" interactive "$d" --port "$3" >/tmp/hs_server_$3.log 2>&1 & echo $! > /tmp/hs_server_$3.pid; sleep 4; echo "up on $3 (pid $(cat /tmp/hs_server_$3.pid))";;
  stop) kill $(cat /tmp/hs_server_$2.pid) 2>/dev/null; rm -f /tmp/hs_server_$2.pid; echo stopped;;
esac
