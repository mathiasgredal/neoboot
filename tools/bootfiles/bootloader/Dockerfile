FROM alpine:latest as dist
COPY /wasm /wasm
RUN ls -la /wasm
RUN dd if=/dev/zero of=/hello2 bs=5M count=1
