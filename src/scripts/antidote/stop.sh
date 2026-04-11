#!/bin/bash

echo "Stopping antidote cluster..."
docker compose -f docker/antidote/docker-compose.yml down
echo "antidote cluster stopped."