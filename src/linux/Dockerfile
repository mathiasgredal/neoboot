### BASE-LAYER ###
FROM debian:sid-slim AS base

# Set the build arguments
ARG LINUX_ARCHIVE

# Install download tools
RUN apt-get update && apt-get install -y curl

# Clone the linux repository
WORKDIR /linux
RUN curl -L $LINUX_ARCHIVE | tar -xz --strip-components=1

# Install the base dependencies
RUN apt-get install -y \
    bison \ 
    efitools \
    flex \
    bc \
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
COPY patches/*.patch /linux/patches/ 
RUN for i in patches/*.patch; do patch -p1 --merge < $i; done

### CROSS-BUILDER-LAYER ###
FROM base AS builder

# Set the build arguments
ARG DEFCONFIG
ARG ARCH
ARG CROSS_COMPILE
ARG REDIS_URL=redis://localhost:6379

ENV CCACHE_REMOTE_STORAGE=${REDIS_URL}
ENV PATH=/usr/lib/ccache:$PATH

# Build linux
RUN make ARCH="$ARCH" CROSS_COMPILE="ccache $CROSS_COMPILE" $DEFCONFIG
RUN make ARCH="$ARCH" CROSS_COMPILE="ccache $CROSS_COMPILE" -j4

### EMULATOR-LAYER ###
FROM debian:bookworm-slim AS dist

RUN mkdir -p /linux

# Copy the linux binary from the builder layer
COPY --from=builder /linux/.config /linux/System.map /linux/arch/*/boot/Image /linux/

### COMPILE-COMMANDS ###
FROM builder AS compile-commands
RUN ./scripts/clang-tools/gen_compile_commands.py -o compile_commands.json
