apps/
    bootloader/
        patches/
        BUILD
        Dockerfile
        README.md
    wasm_oss/
    wasm_pro/
    command_line/
    frontend/
libs/
    wasm_spl_lib/
vendor/
    u-boot/
    linux/
build/
    u-boot/
    
infra/
docs/
tests/
tools/
    bazel/

Makefile:
    u-boot:
        1. Clone u-boot and apply patches
        2. Generate patches
        3. Generate compile_commands.json
    
    frontend:
        1. bun watch nextjs
