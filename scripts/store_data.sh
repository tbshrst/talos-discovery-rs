#!/bin/bash

read -r -p "Enter some data: " data
data=$(echo $data | base64)
echo $data

grpcurl -plaintext  -import-path ../api/discovery-api/api -d '{"cluster_id": "this-cluster-stores-data", "client_version": "2.0"}' \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.Hello

grpcurl -plaintext  -import-path ../api/discovery-api/api -d "{\"cluster_id\": \"this-cluster-stores-data\", \"affiliate_id\": \"1\", \"affiliate_data\": \"$data\", \"affiliate_endpoints\": \"\", \"ttl\": \"300s\"}" \
  -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.AffiliateUpdate
