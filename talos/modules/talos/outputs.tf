output "client_configuration" {
  value     = data.talos_client_configuration.this.talos_config
  sensitive = true
}

output "endpoints" {
  value = local.endpoint_ips
}

output "endpoint_urls" {
  value = local.endpoint_urls
}

output "kube_config" {
  value     = talos_cluster_kubeconfig.this.kubeconfig_raw
  sensitive = true
}

output "machine_config" {
  value     = data.talos_machine_configuration.this
  sensitive = true
}

output "image" {
  value = {
    name       = local.full_image_name
    meta       = data.talos_image_factory_urls.this
    extensions = data.talos_image_factory_extensions_versions.that
  }
}

output "schematic_id" {
  value = talos_image_factory_schematic.this
}
