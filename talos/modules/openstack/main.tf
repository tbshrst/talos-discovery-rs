terraform {
  required_version = "~> 1.10"
  required_providers {
    openstack = {
      source  = "terraform-provider-openstack/openstack"
      version = "~> 3.0"
    }
  }
}

locals {
  instances = {
    for k, v in var.instances : k => {
      name        = join("-", compact([var.prefix, v.name]))
      image_name  = v.image_name
      flavor_name = v.flavor_name
    }
  }
}

resource "openstack_compute_instance_v2" "instances" {
  for_each            = local.instances
  name                = each.value.name
  flavor_name         = each.value.flavor_name
  image_id            = data.openstack_images_image_v2.image_info[each.key].id
  stop_before_destroy = false

  network {
    port = openstack_networking_port_v2.this[each.key].id
  }
}

data "openstack_compute_flavor_v2" "flavors" {
  for_each = local.instances
  name     = each.value.flavor_name
}
