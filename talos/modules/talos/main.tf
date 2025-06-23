terraform {
  required_version = "~> 1.8"
  required_providers {
    talos = {
      source  = "siderolabs/talos"
      version = "~> 0.7"
    }
  }
}

locals {
  nodes_controlplane = { for k, n in var.nodes : k => n if n.machine_type == "controlplane" }
  nodes_worker       = { for k, n in var.nodes : k => n if n.machine_type == "worker" }
  endpoint_ips       = [for k, n in local.nodes_controlplane : n.external_ip]
  endpoint_urls      = [for ip in local.endpoint_ips : "https://${ip}:6443"]
  templates_dir      = "${path.module}/templates"
}

resource "talos_machine_secrets" "this" {
  talos_version = var.cluster.talos_version
}

data "talos_client_configuration" "this" {
  cluster_name         = var.cluster.name
  client_configuration = talos_machine_secrets.this.client_configuration
  endpoints            = [for k, n in local.nodes_controlplane : n.external_ip]
  nodes                = [for k, n in var.nodes : n.internal_ip]
}

data "talos_machine_configuration" "this" {
  for_each         = var.nodes
  cluster_name     = var.cluster.name
  cluster_endpoint = local.endpoint_urls[0]
  talos_version    = var.cluster.talos_version
  machine_type     = each.value.machine_type
  machine_secrets  = talos_machine_secrets.this.machine_secrets
  config_patches = each.value.machine_type == "controlplane" ? [
    templatefile("${local.templates_dir}/controlplane.yaml.tftpl", {
      hostname              = each.value.name
      cluster_name          = var.cluster.name
      endpoint_url          = local.endpoint_urls[0]
      internal_ip           = values(local.nodes_controlplane)[0].internal_ip
      discovery_service_url = var.cluster.discovery_service_url
      network_cidr          = var.cluster.network_cidr
      installer_image       = local.installer_image
    })
    ] : [
    templatefile("${local.templates_dir}/worker.yaml.tftpl", {
      hostname              = each.value.name
      endpoint_url          = local.endpoint_urls[0]
      cluster_name          = var.cluster.name
      discovery_service_url = var.cluster.discovery_service_url
      network_cidr          = var.cluster.network_cidr
      installer_image       = local.installer_image
    })
  ]
}

resource "talos_machine_configuration_apply" "this" {
  for_each                    = var.nodes
  node                        = each.value.external_ip
  client_configuration        = talos_machine_secrets.this.client_configuration
  machine_configuration_input = data.talos_machine_configuration.this[each.key].machine_configuration
}

resource "talos_machine_bootstrap" "this" {
  depends_on           = [talos_machine_configuration_apply.this]
  node                 = values(local.nodes_controlplane)[0].external_ip
  client_configuration = talos_machine_secrets.this.client_configuration
}

data "talos_cluster_health" "this" {
  depends_on             = [talos_machine_bootstrap.this]
  control_plane_nodes    = [for k, n in local.nodes_controlplane : n.internal_ip]
  worker_nodes           = [for k, n in local.nodes_worker : n.internal_ip]
  client_configuration   = data.talos_client_configuration.this.client_configuration
  endpoints              = data.talos_client_configuration.this.endpoints
  skip_kubernetes_checks = true
  timeouts = {
    read = "5m"
  }
}

resource "talos_cluster_kubeconfig" "this" {
  depends_on           = [data.talos_cluster_health.this]
  client_configuration = talos_machine_secrets.this.client_configuration
  node                 = values(local.nodes_controlplane)[0].external_ip
  timeouts = {
    read = "1m"
  }
}
