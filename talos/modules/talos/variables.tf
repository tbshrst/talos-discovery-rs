variable "nodes" {
  description = "Configuration for cluster nodes"
  type = map(object({
    name         = string
    machine_type = string
    external_ip  = string
    internal_ip  = string
  }))

  validation {
    condition     = alltrue([for node in var.nodes : contains(["controlplane", "worker"], node.machine_type)])
    error_message = "Invalid machine type. Valid types: controlplane, worker."
  }

  validation {
    condition     = alltrue([for node in var.nodes : can(regex("^((25[0-5]|2[0-4][0-9]|[0-1]?[0-9][0-9]?)\\.){3}(25[0-5]|2[0-4][0-9]|[0-1]?[0-9][0-9]?)$", node.internal_ip))])
    error_message = "Must be a valid ip v4 address."
  }
}

variable "cluster" {
  description = "Cluster configuration"
  type = object({
    name                  = string
    discovery_service_url = optional(string, "https://discovery.talos.dev")
    talos_version         = optional(string, "v1.9.1")
    network_cidr          = optional(string, null)
  })

  validation {
    condition     = length(trimspace(var.cluster.discovery_service_url)) > 0
    error_message = "Discovery service address must be set. Openstack: \"http://$FLOATING_IP:$PORT\". Default: \"https://discovery.talos.dev\""
  }

  validation {
    condition     = can(regex("^v(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)$", var.cluster.talos_version))
    error_message = "Invalid version format. Valid example: v1.9.1"
  }

  validation {
    condition     = var.cluster.network_cidr == null || can(cidrsubnets(var.cluster.network_cidr))
    error_message = "Invalid network CIDR."
  }
}

variable "talos_image_info" {
  description = "Installer information for image factory."
  type = object({
    version      = string
    architecture = optional(string)
    platform     = optional(string)
    extensions   = optional(list(string), [])
  })

  default = {
    version      = "v1.9.1"
    architecture = "amd64"
    platform     = "openstack"
    extensions   = [""]
  }

  validation {
    condition     = can(regex("^v(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)$", var.talos_image_info.version))
    error_message = "Invalid version format. Valid example: v1.9.1"
  }

  validation {
    condition     = var.talos_image_info.platform == "openstack"
    error_message = "Currently only support for openstack."
  }
}
