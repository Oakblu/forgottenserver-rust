use forgottenserver_common::position::Position;

/// Stub pathfinder: computes a walk path between two positions.
///
/// Real A* is deferred to creature-ai-ticking. For now this returns an empty
/// path so the follow-target state can be wired and tested.
pub struct Pathfinder;

impl Default for Pathfinder {
    fn default() -> Self {
        Pathfinder
    }
}

impl Pathfinder {
    pub fn new() -> Self {
        Pathfinder
    }

    /// Return a walk path (as direction bytes) from `from` to `to`.
    ///
    /// Each byte encodes a cardinal/diagonal direction (0=N, 1=E, 2=S, 3=W,
    /// 4=NE, 5=SE, 6=SW, 7=NW). Returns empty when already at destination.
    pub fn find_path(&self, from: Position, to: Position) -> Vec<u8> {
        if from == to {
            return vec![];
        }
        // Stub: single-step towards target (used only by wiring tests)
        let dx = to.x as i32 - from.x as i32;
        let dy = to.y as i32 - from.y as i32;
        match (dx.signum(), dy.signum()) {
            (0, -1) => vec![0],  // N
            (1, 0) => vec![1],   // E
            (0, 1) => vec![2],   // S
            (-1, 0) => vec![3],  // W
            (1, -1) => vec![4],  // NE
            (1, 1) => vec![5],   // SE
            (-1, 1) => vec![6],  // SW
            (-1, -1) => vec![7], // NW
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_path_same_position_returns_empty() {
        let pf = Pathfinder::new();
        let p = Position::new(100, 100, 7);
        assert!(pf.find_path(p, p).is_empty());
    }

    #[test]
    fn find_path_east_returns_direction_byte() {
        let pf = Pathfinder::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(101, 100, 7);
        let path = pf.find_path(from, to);
        assert!(!path.is_empty());
        assert_eq!(path[0], 1); // E
    }

    #[test]
    fn find_path_north_returns_direction_byte() {
        let pf = Pathfinder::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 99, 7);
        let path = pf.find_path(from, to);
        assert_eq!(path[0], 0); // N
    }
}
