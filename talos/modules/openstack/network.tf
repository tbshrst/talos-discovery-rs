data "openstack_networking_network_v2" "external" {
  name       = "genua-openstack"
  network_id = "02e62130-1a1f-432d-a897-beeb348d2241"
}

resource "openstack_networking_router_v2" "this" {
  name                = join("-", compact([var.prefix, "talos-router"]))
  admin_state_up      = "true"
  external_network_id = data.openstack_networking_network_v2.external.id
}

resource "openstack_networking_network_v2" "talos" {
  name           = join("-", compact([var.prefix, "talos-network"]))
  admin_state_up = "true"
}

resource "openstack_networking_subnet_v2" "this" {
  name       = join("-", compact([var.prefix, "talos-subnet"]))
  network_id = openstack_networking_network_v2.talos.id
  cidr       = "192.168.1.0/24"
  ip_version = 4
  allocation_pool {
    start = "192.168.1.3"
    end   = "192.168.1.254"
  }
}

resource "openstack_networking_router_interface_v2" "int_1" {
  router_id = openstack_networking_router_v2.this.id
  subnet_id = openstack_networking_subnet_v2.this.id
}

resource "openstack_networking_port_v2" "this" {
  for_each              = local.instances
  name                  = "${each.value.name}-talos-port"
  network_id            = openstack_networking_network_v2.talos.id
  admin_state_up        = "true"
  port_security_enabled = "false"

  fixed_ip {
    subnet_id = openstack_networking_subnet_v2.this.id
  }
}

resource "openstack_networking_floatingip_associate_v2" "this" {
  for_each    = local.instances
  floating_ip = openstack_networking_floatingip_v2.this[each.key].address
  port_id     = openstack_networking_port_v2.this[each.key].id
}

resource "openstack_networking_floatingip_v2" "this" {
  for_each = var.instances
  pool     = "genua-openstack"
}

# data "openstack_networking_secgroup_v2" "default" {
#   name = "default"
# }

# resource "openstack_networking_port_secgroup_associate_v2" "port_1" {
#   port_id = openstack_networking_port_v2.port_1.id
#   security_group_ids = [
#     data.openstack_networking_secgroup_v2.default.id,
#   ]
# }
