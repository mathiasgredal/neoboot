# NeoBoot

Welcome to the NeoBoot source code monorepo.

## Getting Started

To get started, make sure you have Docker or Podman installed and running. Then build U-Boot using `make u-boot`, this will clone down the U-Boot source code and place it in `vendor/u-boot`, with the NeoBoot patches applied. It will then build the bootloader for the default arch(x86_64), and the wasm spl, and finally boot them using QEMU using Docker. The resulting binaries are placed in `dist/(arch)`.

If you want to make any changes to the U-Boot source code, it can be helpful to have the compile-commands.json file, which will be used by clangd. To generate this file run `make u-boot-ide`. After making some changes and building it, then build system will export the patch set and place it in the source tree, ensuring deterministic and declarative builds.

To speed up builds using ccache, you can start Redis with `u-boot-redis-up`, the build system will automatically start using it, if your environment is set up correctly.

## Wasm
Designed like Novel Netware

## TODO:
- check that env_net_setup has been called
- convert tcp to netconn for dual udp, tcp support
- create dns function on top of rustdns
- support embedded_io_async and embedded_nal_async traits
- integrate with reqwless(reqwest) and picoserve(axum)
- websockets using tungstenite
- autobahn test suite
- use flatbuffers to create an rpc protocol
- make a python proxy client that communicates with the rpc server
- make a pytest test suite using the proxy client
- enhance performance using mio
- create cli using shellfish with following commands:
    - stats
    - chain
- use binman to bundle wasm and u-boot and support booting from integrated wasm payload
- try on real hardware
- ???
- profit

## Logbook

### 2025-02-20
My TCP bindings weren't slow, it was just that qemu's e1000 driver was slow. Using virtio-net instead of e1000 fixed the issue.
I just wasted the entire day on this.

### 2025-02-21
In addition to u-boot, i will also need to support iPXE as a bootloader runtime.
