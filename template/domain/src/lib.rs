//! Domain layer: the `Item` entity and the `ItemRepository` port that
//! `{{ crate_prefix }}-infrastructure` implements. Zero external deps beyond the
//! platform's DI/error primitives — no Axum, no SeaORM, no serde beyond deriving on
//! plain data.
//!
//! This is a starter vertical slice. Replace `Item`/`ItemRepository` with your own
//! entities, following the same shape: a plain struct plus a `#[component]` trait.

use stano_common::ServiceError;
use stano_di_macros::component;

stano_common::id_type!(ItemId, uuid_v7);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
}

/// Injectable via `#[component]`; resolved as `Arc<dyn ItemRepository>`.
#[component]
pub trait ItemRepository: Send + Sync {
    fn create(&self, name: String) -> Item;
    fn get(&self, id: ItemId) -> Result<Item, ServiceError>;
    fn list(&self) -> Vec<Item>;
}
