# AGENTS.md — {{ project_name }}

Built on [modular-rust-platform](https://github.com/jstano/modular-rust-platform)'s
`stano-*` crates. Read that repo's `AGENTS.md` for full platform semantics (DI container,
route-macro auto-registration, authorization builder). This file states the layering
rules for *this* app.

## Architecture

```
{{ project_name }} (launcher, composition root)
    -> rest_api (HTTP handlers, authorization)
    -> services (business layer)
    -> domain (entities, repository traits — zero external deps)
infrastructure -> domain (repository adapter impls)
```

## Layering rules

| Layer | Error type | May depend on | Must NOT depend on |
|---|---|---|---|
| `domain` | `DomainError`/`stano_common::ServiceError` | `stano-common`, `stano-di`, `stano-di-macros` | anything else |
| `infrastructure` | `anyhow::Error` | `domain`, `stano-common`, `stano-di-macros` (+ SeaORM/HTTP clients when wired) | `services`, `rest_api`, `launcher` |
| `services` | `stano_common::ServiceError` | `domain`, `stano-common`, `stano-di`, `stano-di-macros` | `infrastructure`, `rest_api`, `stano-axum`, `stano-launcher` |
| `rest_api` | `stano_axum::ApiError` | `domain`, `services`, `stano-common`, `stano-starter-rest` | `stano-launcher` (only `launcher/` may depend on it) |
| `launcher` (`main.rs`) | `anyhow::Error` | everything | — (composition root) |

Only the `launcher` binary crate may depend on `stano-launcher`. `rest_api` depends on
`stano-starter-rest`/`stano-axum` only, so it stays a lean, launcher-agnostic adapter.

## DI pattern

`#[component]` on a trait + `#[service(dyn Trait)]` on the impl struct (fields must be
`Arc<T>`), auto-registered via `inventory`. `ApplicationContext::register_all()` (called
in `launcher/src/lib.rs`'s `build_context()`) picks them up; resolve with
`ctx.get_trait::<dyn Foo>()`.

## Route pattern

Handlers use `#[get(path = "...")]`/`#[post(...)]`/etc. from `stano-route-macros`
(re-exported via `stano-starter-rest`) instead of manual router wiring — they
auto-register via `inventory` and generate the OpenAPI spec via `utoipa`.

## Auth pattern

`AuthorizationBuilder::<AppClaims>::new().request_matcher(...).permit_all()/.authenticated()/.has_role(...)`
builds an `AuthorizationLayer` passed into `stano_launcher::run(...)`. See
`rest_api/src/lib.rs`'s `build_authorization`.

## Build & test

```bash
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Replacing the starter `Item` slice

`Item`/`ItemRepository`/`ItemService` in `domain`/`infrastructure`/`services`/`rest_api`
are a placeholder vertical slice proving the platform wiring works end-to-end. Replace
them with your own entities, following the same shape (one `#[component]` trait per
port, one `#[service(dyn Trait)]` impl per adapter).
