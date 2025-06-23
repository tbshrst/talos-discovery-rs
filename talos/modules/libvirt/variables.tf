# variable "libvirt_sock" {
#   description = "Path to local libvirt socket. Must be prefixed with qemu schema."
#   type        = string
#   validation {
#     condition     = startswith(var.libvirt_sock, "qemu:///")
#     error_message = "Invalid path. See https://libvirt.org/uri.html."
#   }
# }

variable "worker_disk_size" {
  description = "Size of worker disk in bytes."
  type        = number
  validation {
    condition     = var.worker_disk_size > 5 * 1024 * 1024 * 1024
    error_message = "Worker disk size must be at least 5 GB."
  }
}

variable "image_path" {
  description = "Path to local talos image."
  type        = string
  validation {
    condition     = can(fileexists(var.image_path)) && fileexists(var.image_path)
    error_message = "The provided path must be a valid local path that exists."
  }
}

variable "libvirt_network_interface" {
  description = "Network configuration for VM."
  type = object({
    bridge = string
  })
}

variable "count_controlplane" {
  description = "Sets the number of control planes."
  type        = number
  default     = 1
  validation {
    condition     = var.count_controlplane > 0
    error_message = "Cluster must at least contain 1 control plane."
  }
}

variable "count_worker" {
  description = "Sets the number of worker nodes."
  type        = number
  default     = 1
  validation {
    condition     = var.count_worker > 0
    error_message = "Cluster must at least contain 1 worker node."
  }
}
