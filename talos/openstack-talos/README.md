# Setup with Terraform
* setup
    * [discovery service](../README.md#member-discovery) is running and its endpoint is known (http://$FLOATING_IP:3000)
    * configure the file `terraform.tfvars`
* get terraform
    ```
    wget https://releases.hashicorp.com/terraform/1.10.3/terraform_1.10.3_linux_amd64.zip && \
        unzip terraform_1.10.3_linux_amd64.zip && \
        chmod +x terraform && \
        export PATH=$PATH:$(pwd)
* init and check changes
    ```
    terraform init
    terraform plan
    ```
* apply setup
    ```
    terraform apply
    ```

# Run
* [Check Talos is running](../README.md#check-talos-is-running)

# Teardown
* `terraform destroy`
* `rm *.img`

# Openstack
* graphical console: `openstack console url show my-instance --novnc` or `https://openstack.genua.de/project/network_topology/`
