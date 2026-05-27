//! Migrated from forgottenserver/src/mounts.h and mounts.cpp
//!
//! Provides the `Mount` struct and `Mounts` registry.

// ---------------------------------------------------------------------------
// Mount
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Mount` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub id: u16,
    pub client_id: u16,
    pub name: String,
    pub speed: i32,
    pub premium: bool,
}

impl Mount {
    pub fn new(
        id: u16,
        client_id: u16,
        name: impl Into<String>,
        speed: i32,
        premium: bool,
    ) -> Self {
        Mount {
            id,
            client_id,
            name: name.into(),
            speed,
            premium,
        }
    }
}

// ---------------------------------------------------------------------------
// Mounts — registry
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Mounts` class.
#[derive(Debug, Default)]
pub struct Mounts {
    mounts: Vec<Mount>,
}

impl Mounts {
    /// Create an empty mount registry.
    pub fn new() -> Self {
        Mounts { mounts: Vec::new() }
    }

    /// Add a mount to the registry.
    pub fn add(&mut self, mount: Mount) {
        self.mounts.push(mount);
    }

    /// Retrieve a mount by server id. Mirrors `getMountByID`.
    pub fn get_mount_by_id(&self, id: u16) -> Option<&Mount> {
        self.mounts.iter().find(|m| m.id == id)
    }

    /// Retrieve a mount by name (case-insensitive). Mirrors `getMountByName`.
    pub fn get_mount_by_name(&self, name: &str) -> Option<&Mount> {
        let lower = name.to_lowercase();
        self.mounts.iter().find(|m| m.name.to_lowercase() == lower)
    }

    /// Retrieve a mount by client id. Mirrors `getMountByClientID`.
    pub fn get_mount_by_client_id(&self, client_id: u16) -> Option<&Mount> {
        self.mounts.iter().find(|m| m.client_id == client_id)
    }

    /// Return all mounts. Mirrors `getMounts`.
    pub fn get_mounts(&self) -> &[Mount] {
        &self.mounts
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mount() -> Mount {
        Mount::new(1, 100, "Deer", 10, false)
    }

    #[test]
    fn test_mount_struct_fields() {
        let m = Mount::new(3, 200, "Neon Spark", 30, true);
        assert_eq!(m.id, 3);
        assert_eq!(m.client_id, 200);
        assert_eq!(m.name, "Neon Spark");
        assert_eq!(m.speed, 30);
        assert!(m.premium);
    }

    #[test]
    fn test_mounts_new_empty() {
        let m = Mounts::new();
        assert!(m.get_mounts().is_empty());
    }

    #[test]
    fn test_mounts_add() {
        let mut m = Mounts::new();
        m.add(sample_mount());
        assert_eq!(m.get_mounts().len(), 1);
    }

    #[test]
    fn test_mounts_get_mount_by_id_found() {
        let mut m = Mounts::new();
        m.add(sample_mount());
        let found = m.get_mount_by_id(1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Deer");
    }

    #[test]
    fn test_mounts_get_mount_by_id_not_found() {
        let m = Mounts::new();
        assert!(m.get_mount_by_id(999).is_none());
    }

    #[test]
    fn test_mounts_get_mount_by_name_found() {
        let mut m = Mounts::new();
        m.add(sample_mount());
        let found = m.get_mount_by_name("Deer");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
    }

    #[test]
    fn test_mounts_get_mount_by_name_case_insensitive() {
        let mut m = Mounts::new();
        m.add(sample_mount());
        // C++ comparison is case-insensitive
        let found = m.get_mount_by_name("deer");
        assert!(found.is_some());
    }

    #[test]
    fn test_mounts_get_mount_by_name_not_found() {
        let m = Mounts::new();
        assert!(m.get_mount_by_name("Unknown").is_none());
    }

    #[test]
    fn test_mounts_get_mount_by_client_id_found() {
        let mut m = Mounts::new();
        m.add(sample_mount());
        let found = m.get_mount_by_client_id(100);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
    }

    #[test]
    fn test_mounts_get_mount_by_client_id_not_found() {
        let m = Mounts::new();
        assert!(m.get_mount_by_client_id(9999).is_none());
    }

    #[test]
    fn test_mounts_get_mounts_returns_all() {
        let mut m = Mounts::new();
        m.add(Mount::new(1, 100, "A", 10, false));
        m.add(Mount::new(2, 101, "B", 20, true));
        assert_eq!(m.get_mounts().len(), 2);
    }

    // --- case-insensitive name lookup: all upper-case ---

    #[test]
    fn test_mounts_get_mount_by_name_all_upper() {
        let mut m = Mounts::new();
        m.add(Mount::new(1, 100, "Deer", 10, false));
        assert!(m.get_mount_by_name("DEER").is_some());
    }

    #[test]
    fn test_mounts_get_mount_by_name_mixed_case() {
        let mut m = Mounts::new();
        m.add(Mount::new(1, 100, "Neon Spark", 30, true));
        assert!(m.get_mount_by_name("nEoN sPaRk").is_some());
    }

    // --- get_mount_by_name returns None for partial match ---

    #[test]
    fn test_mounts_get_mount_by_name_partial_no_match() {
        let mut m = Mounts::new();
        m.add(Mount::new(1, 100, "Deer", 10, false));
        assert!(m.get_mount_by_name("De").is_none());
    }

    // --- get_mount_by_client_id with multiple entries ---

    #[test]
    fn test_mounts_get_mount_by_client_id_second_entry() {
        let mut m = Mounts::new();
        m.add(Mount::new(1, 100, "A", 10, false));
        m.add(Mount::new(2, 200, "B", 20, true));
        let found = m.get_mount_by_client_id(200);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "B");
    }

    // --- premium flag is preserved through registry ---

    #[test]
    fn test_mounts_premium_flag_true() {
        let mut m = Mounts::new();
        m.add(Mount::new(5, 500, "Premium Ride", 50, true));
        let found = m.get_mount_by_id(5).unwrap();
        assert!(found.premium);
    }

    #[test]
    fn test_mounts_premium_flag_false() {
        let mut m = Mounts::new();
        m.add(Mount::new(6, 600, "Free Ride", 10, false));
        let found = m.get_mount_by_id(6).unwrap();
        assert!(!found.premium);
    }

    // --- speed value is preserved through registry ---

    #[test]
    fn test_mounts_speed_value_preserved() {
        let mut m = Mounts::new();
        m.add(Mount::new(7, 700, "Fast", 100, false));
        let found = m.get_mount_by_id(7).unwrap();
        assert_eq!(found.speed, 100);
    }

    // --- get_mounts returns slices in insertion order ---

    #[test]
    fn test_mounts_insertion_order_preserved() {
        let mut m = Mounts::new();
        m.add(Mount::new(3, 300, "Third", 30, false));
        m.add(Mount::new(1, 100, "First", 10, false));
        m.add(Mount::new(2, 200, "Second", 20, false));
        let mounts = m.get_mounts();
        assert_eq!(mounts[0].id, 3);
        assert_eq!(mounts[1].id, 1);
        assert_eq!(mounts[2].id, 2);
    }

    // --- non-free mount (id boundary: u16 max) ---

    #[test]
    fn test_mounts_id_boundary_u16_max() {
        let mut m = Mounts::new();
        m.add(Mount::new(u16::MAX, u16::MAX, "MaxMount", 0, false));
        assert!(m.get_mount_by_id(u16::MAX).is_some());
        assert!(m.get_mount_by_client_id(u16::MAX).is_some());
    }

    // --- get_mount_by_id with multiple mounts, returns None for missing ---

    #[test]
    fn test_mounts_get_mount_by_id_returns_none_when_multiple_loaded() {
        let mut m = Mounts::new();
        m.add(Mount::new(1, 100, "A", 0, false));
        m.add(Mount::new(2, 200, "B", 0, false));
        assert!(m.get_mount_by_id(3).is_none());
    }

    // --- Default trait produces an empty registry (same as new()) ---

    #[test]
    fn test_mounts_default_is_empty() {
        let m = Mounts::default();
        assert!(m.get_mounts().is_empty());
    }
}
