#!/bin/sh
# Removed set -e to prevent container from exiting on error
# set -e 

# Wait for postgres to be ready
echo "Waiting for postgres to be healthy..."
sleep 2

if [ -f "$SPKG" ]; then
    echo "Starting sink with package $SPKG..."
    /app/substreams-sink-sql setup "$PG_DSN" "$SPKG"
    /app/substreams-sink-sql run "$PG_DSN" "$SPKG" -e "$ENDPOINT"
else
    echo "Package $SPKG not found!"
fi

# Keep the container alive for debugging
# echo "Sink process finished or failed. Sleeping indefinitely for debugging..."
# sleep infinity
