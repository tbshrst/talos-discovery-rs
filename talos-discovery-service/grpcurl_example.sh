#!/bin/bash

grpcurl -plaintext  -import-path proto -proto v1alpha1/server/cluster.proto localhost:3000 sidero.discovery.server.Cluster.Hello
