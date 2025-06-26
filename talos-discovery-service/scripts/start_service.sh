#!/bin/bash

podman run -d --restart always -p 3000:3000 -p 3001:3001 ghcr.io/siderolabs/discovery-service:v1.0.10