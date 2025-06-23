output "libvirt_control_plane" {
  value = libvirt_domain.controlplane
}

output "libvirt_worker" {
  value = libvirt_domain.worker
}
