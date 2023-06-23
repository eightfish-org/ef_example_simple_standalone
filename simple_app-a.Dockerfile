FROM docker.io/library/ubuntu:20.04
LABEL description="EightFish:simple_app_standalone"

WORKDIR /eightfish

RUN mkdir -p /eightfish/target/wasm32-wasi/release/

COPY ./spin /usr/local/bin
COPY ./spin-a.toml /eightfish/simple_app_spin.toml
COPY ./target/wasm32-wasi/release/simple.wasm /eightfish/target/wasm32-wasi/release/

