#!/bin/sh

# Note: This script needs to run from inside /api dir
export PROMTAIL_BASE_DIR=$(pwd)

grafana-agent-flow --config.expand-env=true --config.file ./configs/grafana-logs.config.yaml &

cargo run --release
