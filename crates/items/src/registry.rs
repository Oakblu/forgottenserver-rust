use std::collections::HashMap;

use forgottenserver_common::itemloader::ItemGroup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemType {
    pub server_id: u16,
    pub client_id: u16,
    pub group: ItemGroup,
    pub flags: u32,
    pub speed: u16,
    pub weight: u16,
}

#[derive(Debug, Default)]
pub struct ItemsRegistry {
    items: HashMap<u16, ItemType>,
}

impl ItemsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, item: ItemType) {
        self.items.insert(item.server_id, item);
    }

    pub fn get(&self, server_id: u16) -> Option<&ItemType> {
        self.items.get(&server_id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ground_item(server_id: u16) -> ItemType {
        ItemType {
            server_id,
            client_id: 0,
            group: ItemGroup::Ground,
            flags: 0,
            speed: 0,
            weight: 0,
        }
    }

    #[test]
    fn test_registry_new_is_empty() {
        let r = ItemsRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut r = ItemsRegistry::new();
        r.register(ground_item(100));
        assert_eq!(r.len(), 1);
        let item = r.get(100).expect("item 100 should exist");
        assert_eq!(item.server_id, 100);
        assert_eq!(item.group, ItemGroup::Ground);
    }

    #[test]
    fn test_registry_get_unknown_returns_none() {
        let r = ItemsRegistry::new();
        assert!(r.get(9999).is_none());
    }

    #[test]
    fn test_registry_register_overwrites_duplicate() {
        let mut r = ItemsRegistry::new();
        let mut item = ground_item(5);
        item.speed = 100;
        r.register(item);

        let mut item2 = ground_item(5);
        item2.speed = 200;
        r.register(item2);

        assert_eq!(r.len(), 1);
        assert_eq!(r.get(5).unwrap().speed, 200);
    }

    #[test]
    fn test_registry_multiple_items() {
        let mut r = ItemsRegistry::new();
        for id in [1u16, 2, 3, 100, 500] {
            r.register(ground_item(id));
        }
        assert_eq!(r.len(), 5);
        assert!(r.get(100).is_some());
        assert!(r.get(999).is_none());
    }
}
