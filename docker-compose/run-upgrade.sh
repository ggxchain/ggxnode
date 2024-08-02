#!/bin/bash

source .env

# Check if the mount point exists and is owned by 55500:55500
if [ ! -d "$DATA_PATH" ] || [ "$(stat -c %u "$DATA_PATH")" != "55500" ] || [ "$(stat -c %g "$DATA_PATH")" != "55500" ]; then
    echo " Mount point $DATA_PATH does not exist or is not owned by 55500:55500."
    echo " Create the mount point with the following command:"
    echo " sudo -u $USER mkdir -p $DATA_PATH && sudo chown -R 55500:55500 $DATA_PATH"
    exit 1
fi

# Check if container $NODE_NAME exists and if it does, stop and remove it.
if docker inspect "$NODE_NAME" >/dev/null 2>&1; then
    echo " Stopping and removing container $NODE_NAME..."
    docker rm -f "$NODE_NAME" > /dev/null 2>&1 || true
    echo " Container $NODE_NAME stopped and removed."
fi

docker compose -f docker-compose.yml up -d && docker logs "${NODE_NAME}" --follow
