locals {
  output_dir = "${path.root}/talos-config"
}

resource "local_file" "machine_configs" {
  for_each        = data.talos_machine_configuration.this
  filename        = "${local.output_dir}/talos-machine-config-${each.key}.yaml"
  content         = each.value.machine_configuration
  file_permission = "0600"
}

resource "local_file" "talos_config" {
  filename        = "${local.output_dir}/talosconfig"
  content         = data.talos_client_configuration.this.talos_config
  file_permission = "0600"
}

resource "local_file" "kube_config" {
  filename        = "${local.output_dir}/kubeconfig"
  content         = talos_cluster_kubeconfig.this.kubeconfig_raw
  file_permission = "0600"
}
