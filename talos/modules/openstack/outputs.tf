output "instances" {
  value = {
    for key, value in local.instances : key => {
      instance    = openstack_compute_instance_v2.instances[key]
      floating_ip = openstack_networking_floatingip_v2.this[key]
    }
  }
}

output "images" {
  value = data.openstack_images_image_v2.image_info
}

output "subnet_cidr" {
  value = openstack_networking_subnet_v2.this.cidr
}
