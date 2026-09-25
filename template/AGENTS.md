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
persistence -> domain (repository adapter impls)
```

## Layering rules

| Layer | Error type | May depend on | Must NOT depend on |
|---|---|---|---|
| `domain` | `DomainError`/`stano_common::ServiceError` | `stano-common`, `stano-di`, `stano-di-macros` | anything else |
| `persistence` | `anyhow::Error` | `domain`, `stano-common`, `stano-di`, `stano-di-macros`, `stano-seaorm`, `schema-installer`, `schema-sql-generator` (+ HTTP clients if you add other adapters) | `services`, `rest_api`, `launcher` |
| `services` | `stano_common::ServiceError` | `domain`, `stano-common`, `stano-di`, `stano-di-macros` | `persistence`, `rest_api`, `stano-axum`, `stano-launcher` |
| `rest_api` | `stano_axum::ApiError` | `domain`, `services`, `stano-common`, `stano-starter-rest` | `stano-launcher` (only `launcher/` may depend on it) |
| `launcher` (`main.rs`) | `anyhow::Error` | everything | — (composition root) |

Only the `launcher` binary crate may depend on `stano-launcher`. `rest_api` depends on
`stano-starter-rest`/`stano-axum` only, so it stays a lean, launcher-agnostic adapter.

Within a crate, only the `#[component]` trait/port and shared data entities (e.g.
`Item`) should be `pub`. The `#[service(dyn Trait)]` impl struct (e.g.
`ItemServiceImpl`, `SeaOrmItemRepository`) should be `pub(crate)` — nothing outside the
crate needs to name it, since every consumer resolves it via
`ctx.get_trait::<dyn Trait>()`. This gives the same "consumers can't reach
implementation details" guarantee a Java `contracts` module provides, enforced by the
compiler instead of a convention. A crate's own integration tests under `tests/`
compile as a separate crate and can't see `pub(crate)` items, so tests exercising a
`pub(crate)` impl struct directly belong in an inline `#[cfg(test)] mod tests` in
`src/` instead.

This table is enforced, not just documented: `architecture/tests/layering.rs` reads the
resolved `Cargo.toml` dependency graph via `cargo metadata` and fails `cargo test` if any
crate gains a forbidden dependency, or if `launcher` stops wiring one of the layers. If
you deliberately change the architecture, update both the code and this table and the
`forbidden_for`/expected-role lists in that test together.

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

## TDD

Follow test-first (red-green-refactor) for all changes: write a failing test that
captures the new behavior or bug before writing the implementation, watch it fail, then
write the minimal code to make it pass. Refactor with the test green. This applies to
bug fixes too — reproduce the bug as a failing test first, don't just patch the code and
add a test after.

## Build & test

```bash
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

`cargo test` needs Docker running but not `docker compose up` — `launcher/tests/app.rs`
and `persistence/src/lib.rs`'s inline `#[cfg(test)] mod tests` each start their own
throwaway Postgres via `testcontainers`, cleaned up automatically when the test finishes.

## Replacing the starter `Item` slice

`Item`/`ItemRepository`/`ItemService` in `domain`/`persistence`/`services`/`rest_api`
are a placeholder vertical slice proving the platform wiring works end-to-end. Replace
them with your own entities, following the same shape (one `#[component]` trait per
port, one `#[service(dyn Trait)]` impl per adapter).
