#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM debian:bookworm

RUN apt-get update -q && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    ca-certificates \
    # for qemu
    qemu-utils \
    qemu-system-arm \
    qemu-efi-aarch64 \
    ipxe-qemu \
    # for bindgen
    libclang-dev \
    # for sdf
    python3-jinja2 \
    # for test script
    python3-pexpect \
    # for hacking
    bash-completion \
    man \
    sudo \
    && rm -rf /var/lib/apt/lists/*

ENV MICROKIT_SDK=/opt/microkit/sdk

RUN set -eux; \
    mkdir -p $MICROKIT_SDK; \
    cd $MICROKIT_SDK; \
    curl -L \
        "https://github.com/seL4/microkit/releases/download/2.1.0/microkit-sdk-2.1.0-linux-x86-64.tar.gz" \
            | tar -xzf - --strip-components=1; \
    chmod -R a+X .; # HACK

ARG UID
ARG GID

RUN set -eux; \
    if [ $UID -eq 0 ]; then \
        if [ $GID -ne 0 ]; then \
            echo "error: \$UID == 0 but \$GID != 0" >&2; \
            exit 1; \
        fi; \
    else \
        if getent passwd $UID; then \
            echo "error: \$UID $UID already exists" >&2; \
            exit 1; \
        fi; \
        if ! getent group $GID; then \
            groupadd --gid $GID x; \
        fi; \
        useradd --uid $UID --gid $GID --groups sudo --create-home x; \
    fi;

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

USER $UID

RUN set -eux; \
    if [ $UID -ne 0 ]; then \
        curl -sSf https://sh.rustup.rs | \
            bash -s -- -y --no-modify-path --default-toolchain none; \
    fi;

ENV PATH=/home/x/.cargo/bin:$PATH

WORKDIR /work
