# {{ project_name }}

A Rust backend built on the [modular-rust-platform](https://github.com/jstano/modular-rust-platform)
`stano-*` crates (DI, JWT security, Axum HTTP glue, SeaORM, launcher/bootstrap).

## Project structure

```
{{ project_name }}/
├── domain/          # {{ crate_prefix }}-domain: entities + repository traits, zero external deps
├── infrastructure/  # {{ crate_prefix }}-infrastructure: repository adapter impls (in-memory starter)
├── services/        # {{ crate_prefix }}-services: business layer, service traits + impls
├── rest_api/        # {{ crate_prefix }}-rest-api: HTTP handlers, authorization wiring
├── migration/        # {{ crate_prefix }}-migration: SeaORM migrations (starter, not yet wired in)
├── launcher/        # {{ project_name }}: composition root — main.rs, Dockerfile
└── architecture/    # {{ crate_prefix }}-architecture: ArchUnit-style layering tests
```

See `AGENTS.md` for the layering rules and dependency-flow conventions.

## Quick start

```bash
cargo build
cargo run --package {{ project_name }}
```

This works immediately with no setup — it falls back to an in-memory item store and a
hardcoded demo JWT keypair. Before deploying anywhere real:

1. Copy `.env-example` to `.env`.
2. Generate a real ES256 keypair:
   ```bash
   openssl ecparam -name prime256v1 -genkey -noout -out private.pem
   openssl ec -in private.pem -pubout -out public.pem
   ```
   and set `JWT_PRIVATE_KEY`/`JWT_PUBLIC_KEY` in `.env` from those files.
3. Start Postgres (`docker compose up -d postgres`) and switch
   `{{ crate_prefix }}-infrastructure`'s `ItemRepository` impl to a `stano-seaorm`-backed
   adapter, running `migration/`'s schema against it.

## Try it

```bash
curl http://localhost:8080/health
```

The `/items` and `/admin/items` routes require a bearer token (see
`rest_api/src/lib.rs`'s `issue_token` test helper, or wire up a real login endpoint).
Swagger UI is enabled by default at `http://localhost:8080/swagger`.

## Quality checks

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

`cargo test --workspace` includes `architecture/tests/layering.rs`, which asserts the
layering rules in `AGENTS.md` against the actual `Cargo.toml` dependency graph (via
`cargo metadata`) — it fails the build if, say, `domain` ever gains a dependency on
`stano-axum`, or `launcher` stops wiring one of the layers.
