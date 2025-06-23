# Terraform libvirt provider

* WIP: setup currently not working (no volumes attached yet)
* assumption: running locally on client with pcvmhost bridge attached
* adjust libvirt socket in terraform.tfvars
* libvirt module is incompatible with automatic talos image download
  * -> images have to be downloaded manually under https://factory.talos.dev/
  *  -> https://factory.talos.dev/image/9944165e0765a35f754db16c0e3969d84d1968972e6dced70ad139bfc1fc095e/v1.9.1/metal-amd64.qcow2
    * Talos v1.9.1 with extensions: amd-ucode, intel-ucode, kata-containers, qemu-guest-agent
* `NO_PROXY=10.0.0.0/8 terraform apply`