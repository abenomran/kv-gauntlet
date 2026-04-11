#!/bin/bash

echo "Stopping etcd cluster..."
docker compose -f docker/etcd/docker-compose.yml down
echo "etcd cluster stopped."