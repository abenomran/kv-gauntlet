#!/bin/bash

echo "Starting etcd cluster..."
docker compose -f docker/etcd/docker-compose.yml up -d
echo "etcd cluster started."