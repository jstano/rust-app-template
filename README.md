# rust-app-template

A [Copier](https://copier.readthedocs.io/) template for scaffolding new apps that build
on [modular-rust-platform](https://github.com/jstano/modular-rust-platform)'s published
`stano-*` crates (DI, JWT security, Axum HTTP glue, SeaORM, launcher/bootstrap).

Mirrors the layout and tooling of the sibling
[spring-app-template](https://github.com/jstano/spring-app-template), adapted for Rust:
a `copier.yml` at the repo root defines the prompted variables, and `template/` holds the
generated project with Jinja placeholders in file contents. Rust crate names don't need a
Java-style package-directory rename, so there's no post-gen rename script — the generated
project is ready to build as soon as Copier renders it.

## Prerequisites

- [Copier](https://copier.readthedocs.io/en/stable/#installation) (`pipx install copier` or `uv tool install copier`)
- Rust toolchain (stable, edition 2024)
- Docker, if you want to run the local Postgres via `docker-compose.yaml`

## Usage

```bash
copier copy --trust gh:jstano/rust-app-template <output-dir>
# or from a local clone:
copier copy --trust /path/to/rust-app-template <output-dir>
```

`--trust` is required because the template's `_tasks` run `cargo fmt` and `cargo build`
in the generated project after rendering.

## Variables

| Variable | Description | Default |
|---|---|---|
| `project_name` | Workspace/binary crate name, kebab-case (e.g. `my-service`) | — |
| `crate_prefix` | Prefix for member crate names (`{{crate_prefix}}-domain`, `-rest-api`, ...) | `{{ project_name }}` |
| `stano_version` | Version of the published `stano-*` crates to depend on | `0.3.0` |
| `db_name` | PostgreSQL database name | `{{ project_name }}` with `-` replaced by `_` |
| `db_username` | PostgreSQL username | `postgres` |
| `db_password` | PostgreSQL password | *(empty)* |

## What you get

A layered Cargo workspace matching modular-rust-platform's recommended app structure and
its `stano-example-app` reference implementation:

```
<project_name>/
├── domain/          # entities + repository traits, zero external deps
├── infrastructure/  # repository adapter impls
├── services/        # business layer, service traits + impls
├── rest_api/        # HTTP handlers (#[get]/#[post]), authorization wiring
├── migration/        # SeaORM migrations (starter, not wired into infrastructure yet)
└── launcher/        # composition root: main.rs, Dockerfile
```

See `template/README.md` and `template/AGENTS.md` for the generated project's own
quick-start and architecture docs.
