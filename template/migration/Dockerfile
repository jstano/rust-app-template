FROM rust:1.94.1-bookworm AS builder
WORKDIR /usr/src/app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --package {{ crate_prefix }}-migration --bin {{ crate_prefix }}-migration && \
    cp target/release/{{ crate_prefix }}-migration /migration

# Runtime stage - minimal image
FROM debian:bookworm-slim
WORKDIR /usr/src/app
COPY --from=builder /migration ./migration

ENTRYPOINT ["./migration"]
CMD ["up"]
