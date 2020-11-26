FROM rust:1.47.0-alpine3.12

RUN apk update -qq &&\
    apk add --no-cache musl-dev mongodb-tools &&\
    rustup update &&\
    rustup component add clippy rustfmt &&\
    cargo install cargo-watch
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.7.3/wait /wait
RUN chmod a+x /wait
