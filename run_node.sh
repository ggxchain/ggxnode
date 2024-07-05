#!/bin/bash

# This will run single node inside the container

# Ensure there is at least one argument: the executable
if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <executable> [args...]"
  exit 1
fi

EXECUTABLE="$1"
shift 1

# Execute the provided executable with any remaining arguments
"$EXECUTABLE" "$@"
