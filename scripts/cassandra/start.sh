#!/bin/bash

echo "Starting cassandra cluster..."
docker compose -f docker/cassandra/docker-compose.yml up -d
echo "cassandra cluster started."