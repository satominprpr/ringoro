FROM rust:1.47.0-alpine3.12

RUN apk update -qq &&\
    apk add --no-cache musl-dev &&\
    rustup update &&\
    rustup component add clippy rustfmt &&\
    cargo install cargo-watch
