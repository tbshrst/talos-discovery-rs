variable "auth_url" {
  type = string
}

variable "region" {
  type    = string
  default = "RegionOne"
}

variable "user_name" {
  type      = string
  ephemeral = true
}

variable "password" {
  type      = string
  sensitive = true
  ephemeral = true
}

variable "tenant_name" {
  type      = string
  ephemeral = true
}

variable "talos_version" {
  type    = string
  default = "v1.9.2"
}

variable "discovery_service_url" {
  type = string
}

variable "num_controlplanes" {
  description = "Sets the number of control planes."
  type        = number
  default     = 1
  validation {
    condition     = var.num_controlplanes > 0
    error_message = "Cluster must at least contain 1 control plane."
  }
}

variable "num_worker" {
  description = "Sets the number of worker nodes."
  type        = number
  default     = 1
  validation {
    condition     = var.num_worker > 0
    error_message = "Cluster must at least contain 1 worker node."
  }
}
