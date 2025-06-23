output "client_configuration" {
  value     = module.talos.client_configuration
  sensitive = true
}

output "kube_config" {
  value     = module.talos.kube_config
  sensitive = true
}

output "machine_config" {
  value     = module.talos.machine_config
  sensitive = true
}

output "installer_image" {
  value = module.talos.image
}

output "endpoints" {
  value     = module.talos.endpoints
  sensitive = true
}

output "instances" {
  value     = module.openstack.instances
  sensitive = true
}

output "os_images" {
  value     = module.openstack.images
  sensitive = true
}
