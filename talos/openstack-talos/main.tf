terraform {
  required_version = "~> 1.10"
  required_providers {
    openstack = {
      source  = "terraform-provider-openstack/openstack"
      version = "~> 3.0"
    }
    random = {
      source  = "hashicorp/random"
      version = ">= 3.0.0"
    }
  }
}

provider "openstack" {
  user_name   = var.user_name
  tenant_name = var.tenant_name
  password    = var.password
  auth_url    = var.auth_url
  region      = var.region
  insecure    = true # just a workaround, fix asap
}

provider "random" {}

locals {
  username            = data.external.caller_username.result["username"]
  prefix              = "${local.username}-${random_id.this.hex}"
  os_talos_image_name = "${local.username}-${module.talos.image.name}"
}

data "external" "caller_username" {
  program = ["bash", "-c", "echo '{\"username\": \"'$(whoami)'\"}'"]
}

resource "random_id" "this" {
  byte_length = 2
}

module "openstack" {
  source = "../modules/openstack"

  prefix = local.prefix

  instances = merge({
    for i in range(var.num_controlplanes) : format("ctrl-%02d", i) => {
      name        = format("ctrl-%02d", i)
      image_name  = local.os_talos_image_name
      flavor_name = "m1.medium"
    }
    }, {
    for i in range(var.num_worker) : format("worker-%02d", i) => {
      name       = format("worker-%02d", i)
      image_name = local.os_talos_image_name
    }
  })

  images = {
    "talos_1_9" = {
      name             = local.os_talos_image_name
      image_source_url = module.talos.image.meta.urls.disk_image
    },
  }
}

module "talos" {
  source = "../modules/talos"

  cluster = {
    name                  = "talos"
    talos_version         = var.talos_version
    discovery_service_url = var.discovery_service_url
    network_cidr          = module.openstack.subnet_cidr
  }

  talos_image_info = {
    version    = var.talos_version
    platform   = "openstack"
    extensions = ["siderolabs/kata-containers"]
  }

  nodes = {
    for key, value in module.openstack.instances :
    key => {
      name         = value.instance.name
      machine_type = startswith(key, "ctrl") ? "controlplane" : "worker"
      internal_ip  = value.instance.access_ip_v4
      external_ip  = value.floating_ip.address
    }
  }
}
