FROM rust:1.82.0-alpine3.20 AS builder
WORKDIR /app
RUN apk update
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build -p server --locked --release --verbose
RUN mkdir -p build-out/
RUN cp /app/target/release/server build-out/
RUN strip build-out/server

FROM scratch
COPY --from=builder /app/build-out/server .
USER 1000:1000
EXPOSE 42069
ENTRYPOINT ["./server"]
