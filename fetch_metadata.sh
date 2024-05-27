#!/bin/bash -e

# This script runs a both brooklyn and sydney nodes (in --dev), then fetches metadata.
# The metadata is saved to metadata_ggx_brooklyn.scale and metadata_ggx_sydney.scale respectively.

cd $(dirname $0)

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <brooklyn or sydney> <path/to/ggx-node>"
    exit 1
fi

NETWORK="$1"
NODE="$2"

if [ ! -f "$NODE" ]; then
    echo "Error: ggx-node binary not found at $NODE"
    exit 1
fi

function get_metadata() {
    attempts=0
    max_attempts=30
    while sleep 1; do
        (( attempts++ )) || true

        curl -sX POST -H "Content-Type: application/json" --data \
        '{"jsonrpc":"2.0","method":"state_getMetadata", "id": 1}' \
        localhost:9944 | jq .result | cut -d '"' -f 2 | xxd -r -p > $1

        # Check if file is empty
        if [ ! -s "$1" ]; then
            echo "Fetched metadata $1 is empty... retrying (attempt $attempts/$max_attempts)"
            if [ $attempts -ge $max_attempts ]; then
                echo "Failed to fetch metadata $1"
                exit 1
            fi

            continue
        fi

        echo "SUCCESS"
        echo
        return
    done
}

# starts a process then kills it after scale file is fetched
start_and_kill() {
    # Start the command in the background
    $NODE --dev --tmp --rpc-external 1>/dev/null 2>/dev/null &

    # Get its PID
    local pid=$!

    sleep 5  # wait for node to start
    get_metadata node/tests/data/scale/eth_light_client_$NETWORK.scale

    # Then kill it
    kill -9 $pid
}

echo "Starting $NETWORK"
start_and_kill
