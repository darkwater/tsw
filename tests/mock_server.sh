#!/bin/bash
# Mock server that echoes commands received on stdin
echo "Mock server started"
while IFS= read -r line; do
    echo "Received: $line"
    if [ "$line" = "quit" ]; then
        echo "Mock server shutting down"
        exit 0
    fi
done
echo "Mock server EOF"
