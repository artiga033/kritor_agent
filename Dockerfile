FROM rust:1.78-alpine as builder
RUN apk add --no-cache musl-dev protoc
WORKDIR /build
COPY . .
RUN cargo build --release

FROM alpine:3.19
WORKDIR /app
COPY --from=builder /build/target/release/kritor_agent_server ./kritor_agent_server
RUN touch kritor_agent.toml
ENTRYPOINT [ "/app/kritor_agent_server" ]