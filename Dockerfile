FROM alpine:edge

RUN apk update && apk add qemu-system-arm qemu-system-aarch64

WORKDIR /root
