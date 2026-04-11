#!/bin/bash

echo "Stopping cassandra cluster..."
docker compose -f docker/cassandra/docker-compose.yml down
echo "cassandra cluster stopped."