#!/usr/bin/env bash
# Run the oracle on a probe file and print the full output.
# Usage: run.sh <probe.spthy> [extra oracle args]
ORACLE=/home/kamilner/tamarin-cleanroom/wellformedness/oracle/wf_oracle.sh
"$ORACLE" "$@"
