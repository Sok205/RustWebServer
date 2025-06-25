FROM rustlang/rust:nightly as builder

WORKDIR /usr/src/app
COPY . .

RUN rustup target add aarch64-unknown-linux-musl
RUN cargo build --release --target aarch64-unknown-linux-musl

FROM alpine:latest

WORKDIR /app
COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-musl/release/RustWebServer /app/
COPY hello.html /app/
COPY 404.html /app/

EXPOSE 7878

CMD ["./RustWebServer"]