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
    unzip \
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
        "https://productionresultssa7.blob.core.windows.net/actions-results/797488c0-ac6a-4563-9d54-6f671ce673bd/workflow-job-run-b3b4c29a-375e-5034-871e-29135961456d/artifacts/edb0f20b7f086e40497fc5608d28b2b7b4fcc064e2edb84a9949250914ecbc03.zip?rscd=attachment%3B+filename%3D%22microkit-sdk-2.0.1-dev.160%2Ba7f037c-linux-x86-64.zip%22&se=2025-11-21T10%3A35%3A18Z&sig=M6bv3fYi%2Fm8m4ADzEp9%2BhJWcE%2FesCuV4rew5JDQdFzU%3D&ske=2025-11-21T21%3A10%3A01Z&skoid=ca7593d4-ee42-46cd-af88-8b886a2f84eb&sks=b&skt=2025-11-21T09%3A10%3A01Z&sktid=398a6654-997b-47e9-b12b-9515b896b4de&skv=2025-11-05&sp=r&spr=https&sr=b&st=2025-11-21T10%3A25%3A13Z&sv=2025-11-05" \
        -o sdk.zip; \
    unzip sdk.zip; \
    tar -xzf microkit-sdk-*.tar.gz --strip-components=1; \
    chmod -R a+X . # HACK \
    rm sdk.zip;

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
