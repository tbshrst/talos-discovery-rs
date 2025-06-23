#!/bin/bash

grpcurl -plaintext  -import-path ../api/proto -d '{"cluster_id": "this-cluster-stores-data", "client_version": "2.0"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.Hello

grpcurl -plaintext  -import-path ../api/proto -d '{"cluster_id": "this-cluster-stores-data", "affiliate_id": "reader", "affiliate_data": "", "affiliate_endpoints": "", "ttl": "300s"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.AffiliateUpdate

# Watch request
grpcurl -plaintext  -import-path ../api/proto -d '{"cluster_id": "this-cluster-stores-data"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.Watch

# The received data can then be decoded with base64 -d
