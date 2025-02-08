### BASE-LAYER ###
FROM debian:sid-slim AS base

# Set the build arguments
ARG U_BOOT_ARCHIVE

# Install download tools
RUN apt-get update && apt-get install -y curl

# Clone the u-boot repository
WORKDIR /u-boot
RUN curl -L $U_BOOT_ARCHIVE | tar -xz --strip-components=1

# Install the base dependencies
RUN apt-get install -y \
    bison \ 
    efitools \
    flex \
    make \
    patch \
    swig \
    libssl-dev \
    libgnutls28-dev \
    python3-dev \
    python3-setuptools

# Install the cross-compilers
RUN apt-get install -y gcc-x86-64-linux-gnu    
RUN apt-get install -y gcc-arm-linux-gnueabihf
RUN apt-get install -y gcc-aarch64-linux-gnu

# Install ccache
RUN apt-get install -y ccache redis-tools

# Add the patch files and apply them
COPY patches/*.patch /u-boot/patches/ 
RUN for i in patches/*.patch; do patch -p1 --merge < $i; done

### CROSS-BUILDER-LAYER ###
FROM base AS builder

# Set the build arguments
ARG DEFCONFIG=qemu-x86_defconfig
ARG CROSS_COMPILE=x86_64-linux-gnu-
ARG REDIS_URL=redis://localhost:6379

ENV CCACHE_REMOTE_STORAGE=${REDIS_URL}
ENV PATH=/usr/lib/ccache:$PATH

# Build u-boot
RUN make CROSS_COMPILE="ccache $CROSS_COMPILE" $DEFCONFIG
RUN make CROSS_COMPILE="ccache $CROSS_COMPILE" -j4

### EMULATOR-LAYER ###
FROM debian:bookworm-slim AS emulator

# Install qemu
RUN apt-get update && apt-get install -y qemu-system

# Copy the u-boot binary from the builder layer
COPY --from=builder /u-boot/.config /u-boot/u-boot.map* /u-boot/u-boot.bin* /u-boot/u-boot.rom* /u-boot/

### COMPILE-COMMANDS ###
FROM base AS compile-commands

ARG DEFCONFIG

RUN make $DEFCONFIG
RUN make -j4
RUN ./scripts/gen_compile_commands.py -o compile_commands.json
