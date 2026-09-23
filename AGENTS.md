# AGENTS.md — rust-app-template (this repo, not the generated app)

This is the Copier template's own source repo. `template/AGENTS.md` is the style guide
baked into every *generated* app; this file is about maintaining the template itself.

- Files under `template/` are rendered with Jinja by Copier — `{{ project_name }}`,
  `{{ crate_prefix }}`, `{{ stano_version }}`, `{{ db_name }}`, `{{ db_username }}`,
  `{{ db_password }}` are available anywhere in file contents (see `copier.yml`).
- No file/directory renaming is needed (unlike `spring-app-template`'s `__pkg__` +
  `rename_pkg.py` for Java packages) — Cargo crate names are flat strings, so
  `crate_prefix` is substituted directly into each `Cargo.toml`'s `name` field.
- When modular-rust-platform ships a new `stano-*` version, bump `stano_version`'s
  default in `copier.yml` — every dependency line in `template/*/Cargo.toml` reads that
  single variable.
- Verify changes by generating a throwaway project (`copier copy --trust . /tmp/out`)
  and confirming `cargo build`/`cargo test --workspace`/`cargo clippy` succeed in it.
