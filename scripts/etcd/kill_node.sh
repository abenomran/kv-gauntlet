#!/bin/bash

NODE=${1:-etcd2}

echo "Killing node: $NODE"
docker stop $NODE
echo "$NODE stopped."