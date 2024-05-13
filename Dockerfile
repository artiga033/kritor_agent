FROM rust:1.78-bookworm as builder
RUN apt update && apt install -y protobuf-compiler
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt update && apt install -y --no-install-recommends --no-install-suggests \
    openssl && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/kritor_agent_server ./kritor_agent_server
RUN touch kritor_agent.toml
ENTRYPOINT [ "/app/kritor_agent_server" ]