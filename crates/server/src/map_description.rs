use forgottenserver_common::position::Position;
use forgottenserver_world::{World, WorldTile};

pub struct ItemDescriptor {
    pub type_id: u16,
}

pub struct CreatureDescriptor {
    pub id: u32,
}

pub struct TileFlags {
    pub protection_zone: bool,
    pub no_logout: bool,
}

pub struct TileDescription {
    pub pos: Position,
    pub ground: Option<ItemDescriptor>,
    pub top_items: Vec<ItemDescriptor>,
    pub creatures: Vec<CreatureDescriptor>,
    pub flags: TileFlags,
}

impl TileDescription {
    pub fn is_empty(&self) -> bool {
        self.ground.is_none()
            && self.top_items.is_empty()
            && self.creatures.is_empty()
            && !self.flags.protection_zone
            && !self.flags.no_logout
    }
}

pub struct MapDescriptionBuilder<'w> {
    world: &'w World,
}

impl<'w> MapDescriptionBuilder<'w> {
    pub fn new(world: &'w World) -> Self {
        MapDescriptionBuilder { world }
    }

    pub fn build(&self, player_pos: Position) -> Vec<TileDescription> {
        let px = player_pos.x as i32;
        let py = player_pos.y as i32;
        let z = player_pos.z;
        let mut tiles = Vec::with_capacity(18 * 14);
        for y in (py - 6)..=(py + 7) {
            for x in (px - 8)..=(px + 9) {
                let clamped_x = x.clamp(0, u16::MAX as i32) as u16;
                let clamped_y = y.clamp(0, u16::MAX as i32) as u16;
                let pos = Position::new(clamped_x, clamped_y, z);
                let world_tile = self.world.get_tile(pos);
                tiles.push(Self::tile_description(pos, world_tile));
            }
        }
        tiles
    }

    fn tile_description(pos: Position, tile: Option<&WorldTile>) -> TileDescription {
        match tile {
            None => TileDescription {
                pos,
                ground: None,
                top_items: vec![],
                creatures: vec![],
                flags: TileFlags {
                    protection_zone: false,
                    no_logout: false,
                },
            },
            Some(t) => TileDescription {
                pos,
                ground: t.ground_item_id.map(|id| ItemDescriptor { type_id: id }),
                top_items: t
                    .top_item_ids
                    .iter()
                    .map(|&id| ItemDescriptor { type_id: id })
                    .collect(),
                creatures: t
                    .creature_ids
                    .iter()
                    .map(|&id| CreatureDescriptor { id })
                    .collect(),
                flags: TileFlags {
                    protection_zone: t.flags.protection_zone,
                    no_logout: t.flags.no_logout,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_world::{World, WorldTile};

    // -----------------------------------------------------------------------
    // Phase 1
    // -----------------------------------------------------------------------

    #[test]
    fn empty_world_produces_skip_tiles_encoding() {
        let world = World::new();
        let builder = MapDescriptionBuilder::new(&world);
        let player_pos = Position::new(100, 100, 7);
        let tiles = builder.build(player_pos);

        assert_eq!(tiles.len(), 18 * 14, "Viewport must be 18x14 = 252 tiles");
        for tile in &tiles {
            assert!(
                tile.ground.is_none(),
                "Expected no ground item in empty world"
            );
            assert!(
                tile.top_items.is_empty(),
                "Expected no top items in empty world"
            );
            assert!(
                tile.creatures.is_empty(),
                "Expected no creatures in empty world"
            );
            assert!(tile.is_empty(), "All tiles should be empty (skip) markers");
        }
    }

    // -----------------------------------------------------------------------
    // Phase 2
    // -----------------------------------------------------------------------

    #[test]
    fn single_ground_item_at_known_pos_encodes_correctly() {
        let mut world = World::new();
        let item_pos = Position::new(100, 100, 7);
        world.set_tile(
            item_pos,
            WorldTile {
                ground_item_id: Some(500),
                ..WorldTile::default()
            },
        );

        let builder = MapDescriptionBuilder::new(&world);
        let tiles = builder.build(Position::new(100, 100, 7));

        let item_tile = tiles
            .iter()
            .find(|t| t.pos == item_pos)
            .expect("tile must be in viewport");
        assert_eq!(
            item_tile.ground.as_ref().map(|g| g.type_id),
            Some(500),
            "Ground item type_id must match"
        );
    }

    #[test]
    fn creature_in_viewport_included_in_tile_description() {
        let mut world = World::new();
        let creature_pos = Position::new(105, 102, 7);
        world.set_tile(
            creature_pos,
            WorldTile {
                creature_ids: vec![42],
                ..WorldTile::default()
            },
        );

        let builder = MapDescriptionBuilder::new(&world);
        let tiles = builder.build(Position::new(100, 100, 7));

        let creature_tile = tiles
            .iter()
            .find(|t| t.pos == creature_pos)
            .expect("tile in viewport");
        assert_eq!(creature_tile.creatures.len(), 1);
        assert_eq!(creature_tile.creatures[0].id, 42);
    }

    #[test]
    fn tile_outside_viewport_not_included() {
        let mut world = World::new();
        // (115, 100, 7) is 15 tiles east of player at (100, 100), viewport only extends 9 east
        let outside_pos = Position::new(115, 100, 7);
        world.set_tile(
            outside_pos,
            WorldTile {
                ground_item_id: Some(999),
                ..WorldTile::default()
            },
        );

        let builder = MapDescriptionBuilder::new(&world);
        let player_pos = Position::new(100, 100, 7);
        let tiles = builder.build(player_pos);

        let found = tiles.iter().any(|t| t.pos == outside_pos);
        assert!(!found, "Tile outside viewport must not be included");
    }
}
