#!/bin/bash

NODE1=${1:-cassandra2}
NODE2=${2:-cassandra3}

echo "Killing nodes: $NODE1 and $NODE2"
docker stop $NODE1
docker stop $NODE2
echo "$NODE1 and $NODE2 stopped."