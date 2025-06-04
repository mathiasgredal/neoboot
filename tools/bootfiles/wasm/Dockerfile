FROM alpine as builder
RUN echo "Hello World" > /hello

FROM scratch as dist
COPY --from=builder /hello /hello
