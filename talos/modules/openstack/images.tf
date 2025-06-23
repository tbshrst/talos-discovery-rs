data "openstack_images_image_v2" "image_info" {
  for_each         = local.instances
  name             = each.value.image_name
  most_recent      = true
  disk_format      = "raw"
  container_format = "bare"
  depends_on       = [openstack_images_image_v2.download_images]
}

resource "openstack_images_image_v2" "download_images" {
  for_each         = var.images
  name             = each.value.name
  disk_format      = "raw"
  container_format = "bare"
  image_source_url = each.value.image_source_url
  image_cache_path = path.root
  visibility       = "private"
  tags             = ["terraform-managed"]
  decompress       = true
}
