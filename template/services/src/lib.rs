//! Business layer. Thin here (no extra rule beyond delegating to the repository port) —
//! its role is structural: the seam where real business rules (validation, role checks)
//! belong, matching a production service layer's shape.

use std::sync::Arc;

use stano_common::ServiceError;
use stano_di_macros::{component, service};
use {{ crate_prefix | replace('-', '_') }}_domain::{Item, ItemId, ItemRepository};

/// Injectable via `#[component]`; resolved as `Arc<dyn ItemService>`.
#[component]
pub trait ItemService: Send + Sync {
    fn create(&self, name: String) -> Item;
    fn get(&self, id: ItemId) -> Result<Item, ServiceError>;
    fn list(&self) -> Vec<Item>;
}

#[service(dyn ItemService)]
pub struct ItemServiceImpl {
    repo: Arc<dyn ItemRepository>,
}

impl ItemService for ItemServiceImpl {
    fn create(&self, name: String) -> Item {
        self.repo.create(name)
    }

    fn get(&self, id: ItemId) -> Result<Item, ServiceError> {
        self.repo.get(id)
    }

    fn list(&self) -> Vec<Item> {
        self.repo.list()
    }
}
