terraform {
  required_providers {
    libvirt = {
      source  = "dmacvicar/libvirt"
      version = "~> 0.8"
    }
  }
}

resource "libvirt_domain" "controlplane" {
  count       = var.count_controlplane
  name        = format("talos-controlplane%02d", count.index)
  description = "terraform-managed talos controlplane VM"
  qemu_agent  = true
  vcpu        = 2
  memory      = 2048
  machine     = "q35"
  # firmware    =
  cpu {
    mode = "host-passthrough"
  }
  disk {
    volume_id = libvirt_volume.controlplane_image[count.index].id
    scsi      = true
  }
  network_interface {
    bridge         = var.libvirt_network_interface.bridge
    wait_for_lease = true
  }
  lifecycle {
    ignore_changes = [
      nvram,
      disk[0].wwn,
      network_interface[0].addresses,
    ]
  }
}

resource "libvirt_domain" "worker" {
  count       = var.count_worker
  name        = format("talos-worker%02d", count.index)
  description = "terraform-managed talos worker VM"
  qemu_agent  = true
  vcpu        = 1
  memory      = 1024
  machine     = "q35"
  # firmware    =
  cpu {
    mode = "host-passthrough"
  }
  disk {
    volume_id = libvirt_volume.worker_image[count.index].id
    scsi      = true
  }
  disk {
    volume_id = libvirt_volume.worker_data[count.index].id
    scsi      = true
    wwn       = format("000000000000ab%02x", count.index)
  }
  network_interface {
    bridge         = var.libvirt_network_interface.bridge
    wait_for_lease = true
  }
  lifecycle {
    ignore_changes = [
      nvram,
      disk[0].wwn,
      network_interface[0].addresses,
    ]
  }
}

resource "libvirt_volume" "controlplane_image" {
  count            = var.count_controlplane
  name             = format("controlplane-data%02d", count.index)
  base_volume_name = libvirt_volume.talos_image.name
  format           = "qcow2"
  pool             = "default"
}

resource "libvirt_volume" "worker_image" {
  count            = var.count_worker
  name             = format("worker-data%02d", count.index)
  base_volume_name = libvirt_volume.talos_image.name
  format           = "qcow2"
  pool             = "default"
}

resource "libvirt_volume" "worker_data" {
  count  = var.count_worker
  name   = format("worker-data%02dd0", count.index)
  size   = var.worker_disk_size
  format = "qcow2"
  pool   = "default"
}

resource "libvirt_volume" "talos_image" {
  source = var.image_path
  name   = "talos_image"
  pool   = "default"
  format = "qcow2"
}

## yet unsupported in 0.8.1
# resource "libvirt_volume" "talos" {
#   source = "https://factory.talos.dev/image/376567988ad370138ad8b2698212367b8edcb69b5fd68c80be1f2ec7d603b4ba/v1.9.1/kernel-amd64"
#   name   = "talos"
#   pool   = "default"
#   format = "raw"
# }
