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
    url="https://developer.arm.com/-/media/Files/downloads/gnu-a/10.2-2020.11/binrel/gcc-arm-10.2-2020.11-x86_64-aarch64-none-elf.tar.xz"; \
    wget -nv "$url"; \
    tar -xf gcc-arm-*.tar.xz; \
    rm gcc-arm-*.tar.xz; \
    mv gcc-arm-* /opt/gcc-aarch64-none-elf;

ENV PATH=/opt/gcc-aarch64-none-elf/bin:$PATH

ENV MICROKIT_SDK_VERSION=1.2.6

# branch: rust
RUN git clone \
        https://github.com/coliasgroup/microkit.git \
        --branch keep/be3c2149f68b17206d9e03e8b038553c \
        --config advice.detachedHead=false

# branch: rust-microkit
RUN git clone \
        https://github.com/coliasgroup/seL4.git \
        --branch keep/fc80c9ad05d33e77a6b850dae8eb4b83 \
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
