use std::collections::HashMap;

use forgottenserver_common::position::Position;

pub mod house;
pub mod iomap;
pub mod iomapserialize;
pub mod map;

#[derive(Debug, Clone, Default)]
pub struct WorldTileFlags {
    pub protection_zone: bool,
    pub no_logout: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WorldTile {
    pub ground_item_id: Option<u16>,
    pub top_item_ids: Vec<u16>,
    pub creature_ids: Vec<u32>,
    pub flags: WorldTileFlags,
}

/// Definition of a single spawn point loaded from world data.
#[derive(Debug, Clone)]
pub struct SpawnPointDef {
    pub position: Position,
    pub radius: u8,
    pub monster_name: String,
    pub interval_secs: u32,
}

#[derive(Debug, Default)]
pub struct World {
    tiles: HashMap<Position, WorldTile>,
    spawn_points: Vec<SpawnPointDef>,
}

impl World {
    pub fn new() -> Self {
        World {
            tiles: HashMap::new(),
            spawn_points: Vec::new(),
        }
    }

    pub fn get_tile(&self, pos: Position) -> Option<&WorldTile> {
        self.tiles.get(&pos)
    }

    pub fn set_tile(&mut self, pos: Position, tile: WorldTile) {
        self.tiles.insert(pos, tile);
    }

    pub fn add_spawn_point(&mut self, sp: SpawnPointDef) {
        self.spawn_points.push(sp);
    }

    pub fn spawn_points(&self) -> impl Iterator<Item = &SpawnPointDef> {
        self.spawn_points.iter()
    }

    pub fn spawn_point_count(&self) -> usize {
        self.spawn_points.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: u16, y: u16, z: u8) -> Position {
        Position::new(x, y, z)
    }

    #[test]
    fn world_spawn_point_count_empty() {
        let w = World::new();
        assert_eq!(w.spawn_point_count(), 0);
    }

    #[test]
    fn world_add_spawn_point_increments_count() {
        let mut w = World::new();
        w.add_spawn_point(SpawnPointDef {
            position: pos(100, 100, 7),
            radius: 3,
            monster_name: "Rat".to_string(),
            interval_secs: 60,
        });
        assert_eq!(w.spawn_point_count(), 1);
    }

    #[test]
    fn world_spawn_points_iterator_yields_all() {
        let mut w = World::new();
        w.add_spawn_point(SpawnPointDef {
            position: pos(100, 100, 7),
            radius: 3,
            monster_name: "Rat".to_string(),
            interval_secs: 60,
        });
        w.add_spawn_point(SpawnPointDef {
            position: pos(200, 200, 7),
            radius: 5,
            monster_name: "Orc".to_string(),
            interval_secs: 120,
        });
        let names: Vec<&str> = w
            .spawn_points()
            .map(|sp| sp.monster_name.as_str())
            .collect();
        assert_eq!(names, vec!["Rat", "Orc"]);
    }
}
