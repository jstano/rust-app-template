//! In-memory `ItemRepository` adapter — stands in for a real database-backed adapter
//! (e.g. `stano-seaorm`), demonstrating the port/adapter seam. Swap this out once the
//! `migration` crate's schema is wired up, following `stano-seaorm`'s `Mapper<T>` pattern.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use stano_common::ServiceError;
use stano_di_macros::service;
use {{ crate_prefix | replace('-', '_') }}_domain::{Item, ItemId, ItemRepository};

/// In-memory backing store for [`InMemoryItemRepository`]. Registered as a plain
/// instance (via `register_instance`) so `#[service]`'s field-injection can resolve it as
/// a dependency, the same way a real app would inject a DB pool or HTTP client.
#[derive(Clone, Default)]
pub struct ItemStore(Arc<Mutex<HashMap<ItemId, Item>>>);

impl ItemStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<ItemId, Item>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[service(dyn ItemRepository)]
pub struct InMemoryItemRepository {
    store: Arc<ItemStore>,
}

impl ItemRepository for InMemoryItemRepository {
    fn create(&self, name: String) -> Item {
        let item = Item {
            id: ItemId::new(),
            name,
        };
        self.store.lock().insert(item.id, item.clone());
        item
    }

    fn get(&self, id: ItemId) -> Result<Item, ServiceError> {
        self.store
            .lock()
            .get(&id)
            .cloned()
            .ok_or(ServiceError::NotFound)
    }

    fn list(&self) -> Vec<Item> {
        self.store.lock().values().cloned().collect()
    }
}
