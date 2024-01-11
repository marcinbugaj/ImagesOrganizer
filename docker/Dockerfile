FROM ubuntu:22.04

RUN apt update && apt install -y npm curl build-essential gcc-mingw-w64-x86-64

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

RUN /root/.cargo/bin/rustup target add x86_64-pc-windows-gnu
