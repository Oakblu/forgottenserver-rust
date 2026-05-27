use forgottenserver_common::position::Position;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Town {
    pub id: u32,
    pub name: String,
    pub temple_pos: Position,
}

impl Town {
    pub fn new(id: u32, name: impl Into<String>, temple_pos: Position) -> Self {
        Self {
            id,
            name: name.into(),
            temple_pos,
        }
    }
}

#[derive(Debug, Default)]
pub struct Towns {
    towns: HashMap<u32, Town>,
}

impl Towns {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_town(&mut self, town: Town) {
        self.towns.insert(town.id, town);
    }

    pub fn get_town(&self, id: u32) -> Option<&Town> {
        self.towns.get(&id)
    }

    pub fn get_town_by_name(&self, name: &str) -> Option<&Town> {
        self.towns
            .values()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }

    pub fn count(&self) -> usize {
        self.towns.len()
    }

    pub fn all_towns(&self) -> Vec<&Town> {
        self.towns.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pos(x: u16, y: u16, z: u8) -> Position {
        Position { x, y, z }
    }

    #[test]
    fn town_new_stores_fields() {
        let t = Town::new(1, "Thais", make_pos(160, 54, 7));
        assert_eq!(t.id, 1);
        assert_eq!(t.name, "Thais");
        assert_eq!(t.temple_pos.x, 160);
    }

    #[test]
    fn towns_new_is_empty() {
        let towns = Towns::new();
        assert_eq!(towns.count(), 0);
    }

    #[test]
    fn add_and_get_town() {
        let mut towns = Towns::new();
        towns.add_town(Town::new(1, "Thais", make_pos(160, 54, 7)));
        let t = towns.get_town(1).unwrap();
        assert_eq!(t.name, "Thais");
    }

    #[test]
    fn get_unknown_town_returns_none() {
        let towns = Towns::new();
        assert!(towns.get_town(99).is_none());
    }

    #[test]
    fn get_town_by_name_case_insensitive() {
        let mut towns = Towns::new();
        towns.add_town(Town::new(1, "Thais", make_pos(160, 54, 7)));
        let t = towns.get_town_by_name("THAIS").unwrap();
        assert_eq!(t.id, 1);
    }

    #[test]
    fn get_town_by_name_unknown_returns_none() {
        let towns = Towns::new();
        assert!(towns.get_town_by_name("Venore").is_none());
    }

    #[test]
    fn count_increments_on_add() {
        let mut towns = Towns::new();
        towns.add_town(Town::new(1, "Thais", make_pos(160, 54, 7)));
        towns.add_town(Town::new(2, "Venore", make_pos(216, 118, 7)));
        assert_eq!(towns.count(), 2);
    }

    #[test]
    fn add_duplicate_id_replaces() {
        let mut towns = Towns::new();
        towns.add_town(Town::new(1, "Thais", make_pos(160, 54, 7)));
        towns.add_town(Town::new(1, "Thais Updated", make_pos(161, 55, 7)));
        assert_eq!(towns.count(), 1);
        assert_eq!(towns.get_town(1).unwrap().name, "Thais Updated");
    }

    #[test]
    fn all_towns_returns_all() {
        let mut towns = Towns::new();
        towns.add_town(Town::new(1, "Thais", make_pos(160, 54, 7)));
        towns.add_town(Town::new(2, "Venore", make_pos(216, 118, 7)));
        assert_eq!(towns.all_towns().len(), 2);
    }

    #[test]
    fn temple_pos_accessible() {
        let mut towns = Towns::new();
        towns.add_town(Town::new(3, "Carlin", make_pos(32, 31, 7)));
        let t = towns.get_town(3).unwrap();
        assert_eq!(t.temple_pos.z, 7);
    }
}
