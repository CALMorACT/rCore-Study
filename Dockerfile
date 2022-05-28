FROM ubuntu:latest
LABEL maintainer="holakk" \
    version="1.0" \
    description="ubuntu latest(22.04) with tools for tsinghua's rCore-Tutorial-V3 based on dinghao188's"

# install some dependencies
RUN set -x \
    && apt update \
    && apt install -y curl wget autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
    gawk build-essential bison flex texinfo gperf libtool patchutils bc xz-utils \
    zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev git tmux python3 

RUN set -x; \
    RUSTUP='/root/rustup.sh'\
    && cd $HOME \
    && curl https://sh.rustup.rs -sSf > $RUSTUP && chmod +x $RUSTUP \
    && $RUSTUP -y --default-toolchain nightly --profile minimal

# install qemu and riscv64 target
RUN apt install qemu qemu-system-misc

# change to aliyun apt mirror
RUN  sed -i s@/archive.ubuntu.com/@/mirrors.aliyun.com/@g /etc/apt/sources.list

#for chinese network
RUN set -x; \
    APT_CONF='/etc/apt/sources.list'; \
    CARGO_CONF='/root/.cargo/config'; \
    BASHRC='/root/.bashrc' \
    && echo 'export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static' >> $BASHRC \
    && echo 'export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup' >> $BASHRC \
    && touch $CARGO_CONF \
    && echo '[source.crates-io]' > $CARGO_CONF \
    && echo "replace-with = 'ustc'" >> $CARGO_CONF \
    && echo '[source.ustc]' >> $CARGO_CONF \
    && echo 'registry = "git://mirrors.ustc.edu.cn/crates.io-index"' >> $CARGO_CONF

RUN wget "https://static.dev.sifive.com/dev-tools/freedom-tools/v2020.12/riscv64-unknown-elf-toolchain-10.2.0-2020.12.8-x86_64-linux-ubuntu14.tar.gz"