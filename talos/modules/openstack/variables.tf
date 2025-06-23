variable "instances" {
  description = "Instances running in this setup images on Openstack. Needs to be at least 1 machine."
  type = map(object({
    name        = string
    image_name  = string
    flavor_name = optional(string, "m1.small")
  }))
}

variable "images" {
  description = "Available images on Openstack. Currently only compressed raw images supported."
  type = map(object({
    name             = string
    image_source_url = string
  }))
  default = {}
}

variable "prefix" {
  description = "Unique prefix for Openstack resources."
  type        = string
  default     = null
}
