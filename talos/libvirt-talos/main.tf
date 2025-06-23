terraform {
  required_providers {
    libvirt = {
      source  = "dmacvicar/libvirt"
      version = "~> 0.8"
    }
  }
}

provider "libvirt" {
  uri = "qemu:///session?socket=/run/user/3042/libvirt/libvirt-sock"
}

locals {
  username = data.external.caller_username.result["username"]
}

data "external" "caller_username" {
  program = ["bash", "-c", "echo '{\"username\": \"'$(whoami)'\"}'"]
}

module "libvirt" {
  source = "../modules/libvirt"

  image_path         = "/PATH/TO/metal-amd64.qcow2"
  count_controlplane = 2
  count_worker       = 2
  worker_disk_size   = 10 * 1024 * 1024 * 1024
  libvirt_network_interface = {
    bridge = "pcvmhost"
  }
}

module "talos" {
  source = "../modules/talos"

  cluster = {
    name          = "${local.username}-talos-cluster"
    endpoint_ip   = [for idx, addr in module.libvirt.libvirt_control_plane[*].network_interface[*].addresses[0] : addr[0] if !can(regex(":", addr))][0]
    talos_version = "v1.9.1"
  }

  nodes = merge({
    for idx, vm in module.libvirt.libvirt_control_plane : vm.name => {
      ip           = [for addr in vm.network_interface[*].addresses[0] : addr if !can(regex(":", addr))][0]
      machine_type = "controlplane"
    }
    }, {
    for idx, vm in module.libvirt.libvirt_worker : vm.name => {
      ip           = [for addr in vm.network_interface[*].addresses[0] : addr if !can(regex(":", addr))][0]
      machine_type = "worker"
    }
  })
}


output "nodes" {
  value = merge({
    for idx, vm in module.libvirt.libvirt_control_plane : vm.name => {
      ip           = [for addr in vm.network_interface[*].addresses[0] : addr if !can(regex(":", addr))][0]
      machine_type = "controlplane"
    }
    }, {
    for idx, vm in module.libvirt.libvirt_worker : vm.name => {
      ip           = [for addr in vm.network_interface[*].addresses[0] : addr if !can(regex(":", addr))][0]
      machine_type = "worker"
    }
  })
}

output "endpoint" {
  value = [for idx, addr in module.libvirt.libvirt_control_plane[*].network_interface[*].addresses[0] : addr[0] if !can(regex(":", addr))][0]
}
