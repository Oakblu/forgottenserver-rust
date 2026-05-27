use forgottenserver_common::position::Position;

/// Holds the set of creature IDs that are spectating a given area.
/// Mirrors C++ `Spectators` container (creature pointers replaced with u32 IDs).
#[derive(Debug, Default, Clone)]
pub struct Spectators {
    creature_ids: Vec<u32>,
    player_ids: Vec<u32>,
}

impl Spectators {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_creature(&mut self, id: u32, is_player: bool) {
        if is_player {
            if !self.player_ids.contains(&id) {
                self.player_ids.push(id);
            }
        } else if !self.creature_ids.contains(&id) {
            self.creature_ids.push(id);
        }
    }

    pub fn remove_creature(&mut self, id: u32) {
        self.creature_ids.retain(|&c| c != id);
        self.player_ids.retain(|&p| p != id);
    }

    pub fn has_creature(&self, id: u32) -> bool {
        self.creature_ids.contains(&id) || self.player_ids.contains(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.creature_ids.is_empty() && self.player_ids.is_empty()
    }

    pub fn count(&self) -> usize {
        self.creature_ids.len() + self.player_ids.len()
    }

    /// All creature IDs (both monsters/npcs and players).
    pub fn all_ids(&self) -> Vec<u32> {
        let mut ids = self.creature_ids.clone();
        ids.extend_from_slice(&self.player_ids);
        ids
    }

    /// Player IDs only.
    pub fn player_ids(&self) -> &[u32] {
        &self.player_ids
    }

    /// Non-player creature IDs only.
    pub fn creature_ids(&self) -> &[u32] {
        &self.creature_ids
    }

    /// Filter spectators to those within Chebyshev range of `center` from a position list.
    /// Since we store only IDs (not positions), the caller supplies a lookup closure.
    pub fn filter_by_range<F>(&self, center: &Position, range: u32, pos_of: F) -> Vec<u32>
    where
        F: Fn(u32) -> Option<Position>,
    {
        self.all_ids()
            .into_iter()
            .filter(|&id| {
                pos_of(id).is_some_and(|p| {
                    p.z == center.z
                        && p.x.abs_diff(center.x) <= range as u16
                        && p.y.abs_diff(center.y) <= range as u16
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let s = Spectators::new();
        assert!(s.is_empty());
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn add_non_player_creature() {
        let mut s = Spectators::new();
        s.add_creature(1, false);
        assert_eq!(s.count(), 1);
        assert!(s.has_creature(1));
        assert_eq!(s.creature_ids(), &[1]);
        assert!(s.player_ids().is_empty());
    }

    #[test]
    fn add_player() {
        let mut s = Spectators::new();
        s.add_creature(2, true);
        assert_eq!(s.count(), 1);
        assert!(s.has_creature(2));
        assert_eq!(s.player_ids(), &[2]);
        assert!(s.creature_ids().is_empty());
    }

    #[test]
    fn add_duplicate_is_idempotent() {
        let mut s = Spectators::new();
        s.add_creature(1, false);
        s.add_creature(1, false);
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn remove_creature() {
        let mut s = Spectators::new();
        s.add_creature(1, false);
        s.remove_creature(1);
        assert!(s.is_empty());
        assert!(!s.has_creature(1));
    }

    #[test]
    fn remove_player() {
        let mut s = Spectators::new();
        s.add_creature(10, true);
        s.remove_creature(10);
        assert!(s.is_empty());
    }

    #[test]
    fn remove_unknown_is_no_op() {
        let mut s = Spectators::new();
        s.remove_creature(999);
        assert!(s.is_empty());
    }

    #[test]
    fn all_ids_includes_both() {
        let mut s = Spectators::new();
        s.add_creature(1, false);
        s.add_creature(2, true);
        let ids = s.all_ids();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn filter_by_range_includes_nearby() {
        let mut s = Spectators::new();
        s.add_creature(1, false);
        // Add a second id so the lookup closure exercises both arms of its
        // `if/else` (matching id and non-matching id).
        s.add_creature(2, false);
        let center = Position {
            x: 100,
            y: 100,
            z: 7,
        };
        let result = s.filter_by_range(&center, 5, |id| {
            if id == 1 {
                Some(Position {
                    x: 102,
                    y: 103,
                    z: 7,
                })
            } else {
                None
            }
        });
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn filter_by_range_excludes_far() {
        let mut s = Spectators::new();
        s.add_creature(1, false);
        let center = Position {
            x: 100,
            y: 100,
            z: 7,
        };
        let result = s.filter_by_range(&center, 5, |_| {
            Some(Position {
                x: 200,
                y: 200,
                z: 7,
            })
        });
        assert!(result.is_empty());
    }

    #[test]
    fn filter_by_range_excludes_different_floor() {
        let mut s = Spectators::new();
        s.add_creature(1, false);
        let center = Position {
            x: 100,
            y: 100,
            z: 7,
        };
        let result = s.filter_by_range(&center, 100, |_| {
            Some(Position {
                x: 100,
                y: 100,
                z: 6,
            })
        });
        assert!(result.is_empty());
    }

    #[test]
    fn add_duplicate_player_is_idempotent() {
        // Covers the duplicate-player branch in `add_creature` (when
        // `player_ids.contains(&id)` is true and push is skipped). Mirrors the
        // dedup semantics of C++ `SpectatorVec::addSpectators` for players.
        let mut s = Spectators::new();
        s.add_creature(42, true);
        s.add_creature(42, true);
        assert_eq!(s.count(), 1);
        assert_eq!(s.player_ids(), &[42]);
        assert!(s.creature_ids().is_empty());
    }

    #[test]
    fn filter_by_range_excludes_unknown_position() {
        // Covers the `pos_of` returning `None` branch inside `is_some_and`.
        // When the caller's lookup cannot resolve an ID's position, the
        // spectator is excluded rather than crashing.
        let mut s = Spectators::new();
        s.add_creature(1, false);
        s.add_creature(2, true);
        let center = Position {
            x: 100,
            y: 100,
            z: 7,
        };
        // Lookup returns None for every id.
        let result = s.filter_by_range(&center, 5, |_| None);
        assert!(result.is_empty());
    }
}
