FROM ubuntu:18.04

RUN apt-get update \
 && apt-get install --no-install-recommends --yes \
        make gcc-arm-none-eabi libnewlib-arm-none-eabi \
 && apt-get clean \
 && rm -rf /root/.cache \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /carbonite

CMD make

