FROM rust:1.68.0-alpine as build

WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release

FROM alpine:3.17 as base
WORKDIR /app
EXPOSE 3000
COPY --from=build /usr/src/myapp/target/release/echo /app

CMD ["./echo"]