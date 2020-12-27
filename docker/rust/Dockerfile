FROM rust:1.47.0-slim

RUN apt-get update -y &&\
    apt-get upgrade -y &&\
    apt-get install -y lsb-release &&\
    rustup update &&\
    rustup component add clippy rustfmt &&\
    cargo install cargo-watch
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.7.3/wait /wait
RUN chmod a+x /wait
