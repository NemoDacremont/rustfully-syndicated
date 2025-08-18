FROM rust:1-alpine3.22 AS builder

WORKDIR /build

# ENV OPENSSL_STATIC=1
ENV RUSTFLAGS="-C target-feature=-crt-static"

# Required for build
RUN apk add openssl-dev openssl-libs-static musl-dev

COPY Cargo.* .

# RUN mkdir -p src && \
#     echo "fn main() {}" > src/main.rs && \
#     cargo build --target x86_64-unknown-linux-musl --release && \
#     rm -rf ./src/ target/release/deps/rustfully-syndicated* target/release/rustfully-syndicated*

COPY ./src ./src/

RUN cargo build --release --target x86_64-unknown-linux-musl

# idk really what's going on, but it won't work with regular alpine:3.22
FROM rust:alpine

WORKDIR /app

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/rustfully-syndicated .

CMD ["/app/rustfully-syndicated"]
