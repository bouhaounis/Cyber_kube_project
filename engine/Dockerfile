FROM rust:1.75-slim AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY engine/Cargo.toml engine/Cargo.toml
COPY engine/src engine/src
RUN cargo build --release -p cyber-kube-engine

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /src/target/release/cyber-kube-engine /app/cyber-kube-engine
EXPOSE 7000
ENTRYPOINT ["/app/cyber-kube-engine"]

