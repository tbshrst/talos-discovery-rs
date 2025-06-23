FROM mcr.microsoft.com/vscode/devcontainers/base:latest

ARG TOFU_VERSION=1.8.7
ARG TERRAFORM_VERSION=1.10.3
ARG TALOSCTL_VERSION=1.9.1
ARG KUBECTL_VERSION=1.32.0

ENV http_proxy=http://tst-proxy.genua.de:8888
ENV https_proxy=http://tst-proxy.genua.de:8888
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

COPY environment /etc/environment
COPY apt.conf /etc/apt/apt.conf

RUN sed -i 's/deb.debian.org/ubuntu-mirror.genua.de/g' /etc/apt/sources.list.d/debian.sources

RUN apt update && apt install -y \
        curl \
        wget \
        unzip \
        podman \
        python3-dev \
        python3-pip \
        python3-openstackclient \
        uidmap \
        tar \
        pandoc \
        texlive-xetex \
        texlive-lang-german

WORKDIR /tmp
RUN wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -O /usr/bin/yq && \
    chmod +x /usr/bin/yq

RUN wget https://github.com/opentofu/opentofu/releases/download/v${TOFU_VERSION}/tofu_${TOFU_VERSION}_amd64.deb && \
    dpkg -i *.deb && \
    rm *.deb

RUN wget https://releases.hashicorp.com/terraform/${TERRAFORM_VERSION}/terraform_${TERRAFORM_VERSION}_linux_amd64.zip && \
    unzip terraform_${TERRAFORM_VERSION}_linux_amd64.zip && \
    install -m 755 terraform /usr/local/bin

RUN wget https://get.helm.sh/helm-v3.16.4-linux-amd64.tar.gz && \
    tar xvzf helm-v3.16.4-linux-amd64.tar.gz && \
    install -m 755 linux-amd64/helm /usr/local/bin

RUN wget -O talosctl https://github.com/siderolabs/talos/releases/download/v${TALOSCTL_VERSION}/talosctl-linux-amd64 && \
    install -m 755 talosctl /usr/local/bin

RUN wget https://dl.k8s.io/release/v${KUBECTL_VERSION}/bin/linux/amd64/kubectl && \
    install -m 755 kubectl /usr/local/bin && \
    mkdir -p /etc/bash_completion.d && \
    kubectl completion bash | tee /etc/bash_completion.d/kubectl > /dev/null && \
    echo 'complete -o default -F __start_kubectl k' >> ~/.bashrc

COPY --from=docker.io/derailed/k9s:v0.32.7 /bin/k9s /usr/local/bin/k9s

RUN curl -Lo /usr/local/bin/skaffold https://storage.googleapis.com/skaffold/releases/latest/skaffold-linux-amd64 && \
    chmod +x /usr/local/bin/skaffold && \
    ln -s /usr/bin/podman /usr/bin/docker

ENV DOCKER_HOST=unix:///run/podman/podman.sock

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -vy && \
    . ~/.bashrc && \
    cargo install cargo-make

RUN rm -r /tmp/*

ENV no_proxy=openstack.genua.de
ENV NO_PROXY=${no_proxy}
ENV OS_CACERT=/usr/local/share/ca-certificates/genua-sys.crt

COPY genua-sys.crt /usr/local/share/ca-certificates/
RUN update-ca-certificates

RUN echo "alias k='kubectl'" >> ~/.bashrc && \
    echo "alias t='talosctl'" >> ~/.bashrc && \
    echo "alias tf='terraform'" >> ~/.bashrc && \
    echo "[[registry]]\nlocation = \"localhost:32000\"\ninsecure = true" > /etc/containers/registries.conf.d/localhost.conf && \
    echo "[[registry]]\nlocation = \"docker.io\"\ninsecure = false" > /etc/containers/registries.conf.d/dockerhub.conf && \
    echo "[[registry]]\nlocation = \"gitlab.genua.de:5005\"\ninsecure = true" > /etc/containers/registries.conf.d/gitlab_genua.conf