#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

FROM trustworthysystems/sel4

WORKDIR /tmp

RUN rm -r *

RUN apt-get update -q && apt-get install -y --no-install-recommends \
    python3-venv \
    musl-tools \
    pandoc \
    texlive-latex-base \
    texlive-latex-extra \
    texlive-fonts-recommended \
    && rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    url="https://developer.arm.com/-/media/Files/downloads/gnu/12.2.rel1/binrel/arm-gnu-toolchain-12.2.rel1-x86_64-aarch64-none-elf.tar.xz"; \
    wget -nv "$url"; \
    tar -xf arm-gnu-toolchain-*.tar.xz; \
    rm arm-gnu-toolchain-*.tar.xz; \
    mv arm-gnu-toolchain-* /opt/gcc-aarch64-none-elf;

ENV PATH=/opt/gcc-aarch64-none-elf/bin:$PATH

ENV MICROKIT_SDK_VERSION=1.2.6

RUN set -eux; \
    git clone \
        https://github.com/seL4/microkit.git \
        --config advice.detachedHead=false; \
    cd microkit; \
    git checkout 35978315953532d3d2aca3faa82eda54d2a9172f;

# TODO
# branch: rust-microkit
RUN git clone \
        https://github.com/coliasgroup/seL4.git \
        --branch keep/1daabe63fd34ae3d348b8e4a4057bd1a \
        --config advice.detachedHead=false \
        microkit/seL4

RUN set -eux; \
    cd microkit; \
    python3.9 -m venv pyenv; \
    ./pyenv/bin/pip install --upgrade pip setuptools wheel; \
    ./pyenv/bin/pip install -r requirements.txt; \
    ./pyenv/bin/pip install sel4-deps; \
    ./pyenv/bin/python3 build_sdk.py --sel4 ./seL4; \
    chmod a+rX release/microkit-sdk-$MICROKIT_SDK_VERSION/bin/microkit; \
    mkdir /opt/microkit; \
    mv release/microkit-sdk-$MICROKIT_SDK_VERSION /opt/microkit; \
    rm -rf $HOME/.cache/pyoxidizer; \
    cd ..; \
    rm -rf microkit;

ENV MICROKIT_SDK=/opt/microkit/microkit-sdk-$MICROKIT_SDK_VERSION

ARG UID
ARG GID

RUN groupadd -f -g $GID x && useradd -u $UID -g $GID -G sudo -m -p x x
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers # for convenience

USER x

RUN curl -sSf https://sh.rustup.rs | \
        bash -s -- -y --no-modify-path --default-toolchain none

ENV PATH=/home/x/.cargo/bin:$PATH

WORKDIR /work
