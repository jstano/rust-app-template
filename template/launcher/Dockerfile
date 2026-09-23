# Stage 1: builder
FROM rust:1.94.1-bookworm AS builder
WORKDIR /usr/src/app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --package {{ project_name }} && \
    cp target/release/{{ project_name }} /{{ project_name }}

# Stage 2: runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app
COPY --from=builder /{{ project_name }} .
EXPOSE 8080
CMD ["./{{ project_name }}"]
