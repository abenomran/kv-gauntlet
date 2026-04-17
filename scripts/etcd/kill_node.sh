#!/bin/bash

NODE=${1:-etcd1}

echo "Killing node: $NODE"
docker stop $NODE
echo "$NODE stopped."