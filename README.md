# NeoBoot

Welcome to the NeoBoot source code monorepo.

## Getting Started

To get started, make sure you have Docker or Podman installed and running. Then build U-Boot using `make u-boot`, this will clone down the U-Boot source code and place it in `vendor/u-boot`, with the NeoBoot patches applied. It will then build the bootloader for the default arch(x86_64), and the wasm spl, and finally boot them using QEMU using Docker. The resulting binaries are placed in `dist/(arch)`.

If you want to make any changes to the U-Boot source code, it can be helpful to have the compile-commands.json file, which will be used by clangd. To generate this file run `make u-boot-ide`. After making some changes and building it, then build system will export the patch set and place it in the source tree, ensuring deterministic and declarative builds.

To speed up builds using ccache, you can start Redis with `u-boot-redis-up`, the build system will automatically start using it, if your environment is set up correctly.


Invalid Opcode (Undefined Opcode)
EIP: 0010:[<020c02c6>] EFLAGS: 00000206
Original EIP :[<c30a52c6>]
EAX: ffffffba EBX: 00000000 ECX: 020c02c4 EDX: d5c9fb31
ESI: 3eff77c0 EDI: 2ef09988 EBP: 2eed8a48 ESP: 2eed8a1c
 DS: 0018 ES: 0018 FS: 0020 GS: 0018 SS: 0018
CR0: 00000033 CR2: 00000000 CR3: 00000000 CR4: 00000000
DR0: 00000000 DR1: 00000000 DR2: 00000000 DR3: 00000000
DR6: ffff0ff0 DR7: 00000400
Stack:
    0x2eed8a5c : 0x2f039b2c
    0x2eed8a58 : 0x00000000
    0x2eed8a54 : 0x00000000
    0x2eed8a50 : 0x000ffde0
    0x2eed8a4c : 0x2ef19a28
    0x2eed8a48 : 0x00000000
    0x2eed8a44 : 0x2f048334
    0x2eed8a40 : 0x2f048334
    0x2eed8a3c : 0x00000000
    0x2eed8a38 : 0x00000000
    0x2eed8a34 : 0x00000000
    0x2eed8a30 : 0x2f049ce4
    0x2eed8a2c : 0x3efa6a66
    0x2eed8a28 : 0x2ef09988
    0x2eed8a24 : 0x3ef3bb57
    0x2eed8a20 : 0x2ef047d0
--->0x2eed8a1c : 0x3ef3bb72
    0x2eed8a18 : 0x00000206
    0x2eed8a14 : 0x00000010
    0x2eed8a10 : 0x020c02c6

You really shined some light on the situation

Can we have stack traces
We have stack traces at home
Stack traces at home: