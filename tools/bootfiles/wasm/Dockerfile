FROM alpine as builder
RUN dd if=/dev/zero of=/hello bs=5M count=1
RUN ls -la /hello

FROM scratch as dist
COPY --from=builder /hello /hello
