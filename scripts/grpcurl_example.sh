#!/bin/bash

grpcurl -plaintext -import-path ../api/discovery-api/api -d '{"cluster_id": "123abc", "client_version": "2.0"}' \
  -proto v1alpha1/server/cluster.proto localhost:80 sidero.discovery.server.Cluster.Hello

grpcurl -plaintext -import-path ../api/discovery-api/api -d '{"cluster_id": "123abc"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.Watch

grpcurl -plaintext -import-path ../api/discovery-api/api -d '{"cluster_id": "123abc", "affiliate_id": "321", "affiliate_data": "", "affiliate_endpoints": "", "ttl": "90s"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.AffiliateUpdate

grpcurl -plaintext -import-path ../api/discovery-api/api -d '{"cluster_id": "123abc", "affiliate_id": "321"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.AffiliateDelete

grpcurl -plaintext -import-path ../api/discovery-api/api -d '{"cluster_id": "123abc"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.List
