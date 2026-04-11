#!/bin/bash

NODE=${1:-cassandra2}

echo "Killing node: $NODE"
docker stop $NODE
echo "$NODE stopped."