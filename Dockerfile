FROM rust:1.77.2-bookworm as build
WORKDIR /usr/src/myapp
COPY . .
RUN apt update -y && apt install mold -y
RUN cargo build --release

FROM debian:bookworm-slim as base
WORKDIR /app
EXPOSE 3000
ENV RUST_LOG=info
COPY --from=build /usr/src/myapp/target/release/echo /app

CMD ["./echo"]
