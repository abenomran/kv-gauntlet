#!/bin/bash

NODE=${1:-antidote2}

echo "Killing node: $NODE"
docker stop $NODE
echo "$NODE stopped."