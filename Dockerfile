FROM rust:1.73.0-buster as build

WORKDIR /usr/src/myapp
COPY . .
RUN apt update -y && apt install mold -y
RUN cargo build --release

FROM debian:buster-slim as base
WORKDIR /app
EXPOSE 3000
ENV RUST_LOG=info
COPY --from=build /usr/src/myapp/target/release/echo /app

CMD ["./echo"]
