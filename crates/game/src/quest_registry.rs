/// A single mission within a quest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionDef {
    pub name: String,
    pub description: String,
}

/// A quest definition, including its missions.
#[derive(Debug, Clone)]
pub struct QuestDef {
    pub id: u16,
    pub name: String,
    pub completed: bool,
    pub missions: Vec<MissionDef>,
}

/// Registry of quest definitions.
#[derive(Debug, Default)]
pub struct QuestRegistry {
    quests: Vec<QuestDef>,
}

impl QuestRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, quest: QuestDef) {
        self.quests.push(quest);
    }

    /// Return all quests (e.g., filtered for the current player in a real impl).
    pub fn list_for_player(&self) -> &[QuestDef] {
        &self.quests
    }

    /// Return the quest definition for `quest_id`, or `None` if not found.
    pub fn mission_info(&self, quest_id: u16) -> Option<&QuestDef> {
        self.quests.iter().find(|q| q.id == quest_id)
    }

    /// Called when a player's storage value changes. Marks matching quests complete.
    /// Convention: quest with `id == key as u16` is completed when `value > 0`.
    pub fn on_storage_change(&mut self, _player_id: u32, key: u32, value: i32) {
        for quest in &mut self.quests {
            if quest.id as u32 == key && value > 0 {
                quest.completed = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_quest() -> QuestDef {
        QuestDef {
            id: 1,
            name: "The Rookie Quest".to_string(),
            completed: false,
            missions: vec![MissionDef {
                name: "Speak to the Sage".to_string(),
                description: "Find the Sage in the temple.".to_string(),
            }],
        }
    }

    #[test]
    fn list_for_player_empty_initially() {
        let reg = QuestRegistry::new();
        assert!(reg.list_for_player().is_empty());
    }

    #[test]
    fn register_and_list_quest() {
        let mut reg = QuestRegistry::new();
        reg.register(sample_quest());
        assert_eq!(reg.list_for_player().len(), 1);
    }

    #[test]
    fn mission_info_found() {
        let mut reg = QuestRegistry::new();
        reg.register(sample_quest());
        let q = reg.mission_info(1);
        assert!(q.is_some());
        assert_eq!(q.unwrap().name, "The Rookie Quest");
    }

    #[test]
    fn mission_info_not_found_returns_none() {
        let reg = QuestRegistry::new();
        assert!(reg.mission_info(99).is_none());
    }

    #[test]
    fn mission_def_fields_accessible() {
        let m = MissionDef {
            name: "Phase 1".to_string(),
            description: "Do things.".to_string(),
        };
        assert_eq!(m.name, "Phase 1");
        assert_eq!(m.description, "Do things.");
    }
}
