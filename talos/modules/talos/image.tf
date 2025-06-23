locals {
  extensions_full_name = [for ext in var.talos_image_info.extensions : ext if ext != ""]
  extensions_name      = [for ext in local.extensions_full_name : split("/", ext)[1]]
  full_image_name      = join("-", concat(["talos", var.talos_image_info.version, var.talos_image_info.platform], compact(local.extensions_name)))
  installer_image      = data.talos_image_factory_urls.this.urls.installer
}

resource "talos_image_factory_schematic" "this" {
  schematic = yamlencode(
    {
      customization = {
        systemExtensions = {
          officialExtensions = local.extensions_full_name
        }
      }
    }
  )
}

data "talos_image_factory_extensions_versions" "that" {
  talos_version = var.talos_image_info.version
  filters = {
    names = local.extensions_full_name
  }
}

data "talos_image_factory_urls" "this" {
  schematic_id  = talos_image_factory_schematic.this.id
  talos_version = var.talos_image_info.version
  architecture  = var.talos_image_info.architecture
  platform      = var.talos_image_info.platform
}
