#!/bin/bash

echo "Starting antidote cluster..."
docker compose -f docker/antidote/docker-compose.yml up -d
echo "antidote cluster started."