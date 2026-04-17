#!/bin/bash
NODE=${1:-antidote2}

# the network this container is on
NETWORK=$(docker inspect -f '{{range $k, $v := .NetworkSettings.Networks}}{{$k}}{{end}}' $NODE)

echo "Partitioning node: $NODE on network: $NETWORK"
docker network disconnect $NETWORK $NODE
echo "$NODE partitioned."