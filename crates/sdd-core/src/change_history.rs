use crate::change::Change;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeHistory {
    pub entries: Vec<HistoryEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub change_id: Uuid,
    pub action: HistoryAction,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub details: HashMap<String, String>,
    pub previous_status: Option<String>,
    pub new_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HistoryAction {
    Created,
    Analyzed,
    Approved,
    Started,
    Resolved,
    Rejected,
    Deferred,
    Modified,
    RolledBack,
}

impl ChangeHistory {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            entries: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn record_creation(&mut self, change: &Change, actor: &str) {
        self.add_entry(
            change.id,
            HistoryAction::Created,
            actor,
            None,
            format!("{:?}", change.status),
            HashMap::new(),
        );
    }

    pub fn record_status_change(
        &mut self,
        change: &Change,
        action: HistoryAction,
        actor: &str,
        previous_status: &str,
    ) {
        self.add_entry(
            change.id,
            action,
            actor,
            Some(previous_status.to_string()),
            format!("{:?}", change.status),
            HashMap::new(),
        );
    }

    pub fn record_modification(
        &mut self,
        change: &Change,
        actor: &str,
        modifications: HashMap<String, String>,
    ) {
        self.add_entry(
            change.id,
            HistoryAction::Modified,
            actor,
            None,
            format!("{:?}", change.status),
            modifications,
        );
    }

    pub fn record_rollback(&mut self, change: &Change, actor: &str, reason: &str) {
        let mut details = HashMap::new();
        details.insert("reason".to_string(), reason.to_string());

        self.add_entry(
            change.id,
            HistoryAction::RolledBack,
            actor,
            Some(format!("{:?}", change.status)),
            "RolledBack".to_string(),
            details,
        );
    }

    fn add_entry(
        &mut self,
        change_id: Uuid,
        action: HistoryAction,
        actor: &str,
        previous_status: Option<String>,
        new_status: String,
        details: HashMap<String, String>,
    ) {
        let entry = HistoryEntry {
            id: Uuid::new_v4(),
            change_id,
            action,
            timestamp: Utc::now(),
            actor: actor.to_string(),
            details,
            previous_status,
            new_status,
        };

        self.entries.push(entry);
        self.updated_at = Utc::now();
    }

    pub fn get_history_for_change(&self, change_id: Uuid) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.change_id == change_id)
            .collect()
    }

    pub fn get_recent_entries(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(limit).collect()
    }

    pub fn get_entries_by_action(&self, action: &HistoryAction) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.action == *action)
            .collect()
    }

    pub fn get_entries_by_actor(&self, actor: &str) -> Vec<&HistoryEntry> {
        self.entries.iter().filter(|e| e.actor == actor).collect()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.updated_at = Utc::now();
    }
}

impl Default for ChangeHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::{Change, ChangeOrigin, ChangeType};

    #[test]
    fn test_history_creation() {
        let history = ChangeHistory::new();
        assert_eq!(history.entry_count(), 0);
    }

    #[test]
    fn test_record_creation() {
        let mut history = ChangeHistory::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );

        history.record_creation(&change, "user");
        assert_eq!(history.entry_count(), 1);
        assert_eq!(history.entries[0].action, HistoryAction::Created);
    }

    #[test]
    fn test_get_history_for_change() {
        let mut history = ChangeHistory::new();
        let change1 = Change::new(
            "CHG-001",
            "Test 1",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );
        let change2 = Change::new(
            "CHG-002",
            "Test 2",
            "Test",
            ChangeType::Architecture,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );

        history.record_creation(&change1, "user");
        history.record_creation(&change2, "user");

        let entries = history.get_history_for_change(change1.id);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_get_recent_entries() {
        let mut history = ChangeHistory::new();
        let change = Change::new(
            "CHG-001",
            "Test",
            "Test",
            ChangeType::Requirement,
            ChangeOrigin::User {
                reason: "Test".to_string(),
            },
        );

        for _ in 0..5 {
            history.record_creation(&change, "user");
        }

        let recent = history.get_recent_entries(3);
        assert_eq!(recent.len(), 3);
    }
}
