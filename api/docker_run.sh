#!/bin/sh

# Note: This script needs to run from inside /api dir
export PROMTAIL_BASE_DIR=$(pwd)
export METRICS_PORT=8001
grafana-agent-flow run ./configs/grafana-logs.config.river

cargo run --release
