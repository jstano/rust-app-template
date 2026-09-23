//! Composition root: wires DI (`#[component]`/`#[service]` auto-registration) and
//! exposes `build_context()` for `main.rs` and the integration tests.

use std::sync::Arc;

use stano_di::application_context::ApplicationContext;
use stano_di::environment::OsEnvironment;
use {{ crate_prefix | replace('-', '_') }}_persistence::ItemStore;

/// Builds the app's `ApplicationContext`: registers the `ItemStore` instance the
/// `#[service]`-generated `InMemoryItemRepository` factory depends on, then picks up
/// every `#[service]`-registered component (in `{{ crate_prefix }}-persistence` and
/// `{{ crate_prefix }}-services`) via `register_all()`.
pub fn build_context() -> Arc<ApplicationContext> {
    let mut ctx = ApplicationContext::new(Arc::new(OsEnvironment::new()));
    ctx.register_instance(Arc::new(ItemStore::default()));
    ctx.register_all();
    ctx.validate()
        .expect("all registered components must resolve");
    Arc::new(ctx)
}
