FROM archlinux/base

RUN pacman -Syu && pacman -S --noconfirm qemu qemu-arch-extra arm-none-eabi-gcc make
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
RUN ~/.cargo/bin/rustup target add --toolchain nightly arm-unknown-linux-gnueabihf

WORKDIR /root
