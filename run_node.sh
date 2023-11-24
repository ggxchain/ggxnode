#!/bin/bash

# Check for at least two arguments: configuration file and executable
if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <config_file> <executable> [args...]"
  exit 1
fi

# The URL to query for checkpoint
URL="https://sync-goerli.beaconcha.in/checkpointz/v1/status"

CONFIG_FILE="$1"
EXECUTABLE="$2"
shift 2

FINALIZED_ROOT=$(curl -s $URL | jq -r '.data.finality.finalized.root')

# Check if the FINALIZED_ROOT is empty or not
if [ -z "$FINALIZED_ROOT" ]; then
    echo "No data found"
else
    # Replace the init_block_root value in the config file
    sed -i "s|init_block_root=.*|init_block_root=\"$FINALIZED_ROOT\"|" "$CONFIG_FILE"
    echo "Updated init_block_root to $FINALIZED_ROOT in $CONFIG_FILE"
fi

# Execute the provided executable with remaining arguments
"$EXECUTABLE" "$@"