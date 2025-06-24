#!/bin/bash

# Client Hello request
echo "Hello request:"
grpcurl -plaintext  -import-path proto -d '{"cluster_id": "123abc", "client_version": "2.0"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.Hello

# Watch request
grpcurl -plaintext  -import-path proto -d '{"cluster_id": "123abc"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.Watch

# AffiliateUpdate request
echo "AffiliateUpdate request:"
grpcurl -plaintext  -import-path proto -d '{"cluster_id": "123abc", "affiliate_id": "321", "affiliate_data": "", "affiliate_endpoints": "", "ttl": "90s"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.AffiliateUpdate

# List Request
grpcurl -plaintext  -import-path proto -d '{"cluster_id": "123abc"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.List
