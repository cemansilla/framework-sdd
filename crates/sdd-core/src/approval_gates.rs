use crate::agent::AgentAction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalGate {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub task_id: Uuid,
    pub agent_id: Uuid,
    pub action: AgentAction,
    pub reason: String,
    pub context: HashMap<String, String>,
    pub status: ApprovalStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub resolution_notes: Option<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    TimedOut,
    Cancelled,
}

impl ApprovalGate {
    pub fn new(
        execution_id: Uuid,
        task_id: Uuid,
        agent_id: Uuid,
        action: AgentAction,
        reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            execution_id,
            task_id,
            agent_id,
            action,
            reason,
            context: HashMap::new(),
            status: ApprovalStatus::Pending,
            created_at: Utc::now(),
            resolved_at: None,
            resolved_by: None,
            resolution_notes: None,
            timeout_seconds: 3600,
        }
    }

    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    pub fn approve(&mut self, resolved_by: &str, notes: Option<&str>) {
        self.status = ApprovalStatus::Approved;
        self.resolved_at = Some(Utc::now());
        self.resolved_by = Some(resolved_by.to_string());
        self.resolution_notes = notes.map(|s| s.to_string());
    }

    pub fn reject(&mut self, resolved_by: &str, notes: Option<&str>) {
        self.status = ApprovalStatus::Rejected;
        self.resolved_at = Some(Utc::now());
        self.resolved_by = Some(resolved_by.to_string());
        self.resolution_notes = notes.map(|s| s.to_string());
    }

    pub fn cancel(&mut self) {
        self.status = ApprovalStatus::Cancelled;
        self.resolved_at = Some(Utc::now());
    }

    pub fn check_timeout(&mut self) -> bool {
        if self.status != ApprovalStatus::Pending {
            return false;
        }

        let elapsed = Utc::now()
            .signed_duration_since(self.created_at)
            .num_seconds();

        if elapsed > self.timeout_seconds as i64 {
            self.status = ApprovalStatus::TimedOut;
            self.resolved_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    pub fn is_pending(&self) -> bool {
        self.status == ApprovalStatus::Pending
    }

    pub fn is_resolved(&self) -> bool {
        matches!(
            self.status,
            ApprovalStatus::Approved
                | ApprovalStatus::Rejected
                | ApprovalStatus::TimedOut
                | ApprovalStatus::Cancelled
        )
    }

    pub fn is_approved(&self) -> bool {
        self.status == ApprovalStatus::Approved
    }
}

#[derive(Debug, Clone)]
pub struct ApprovalManager {
    gates: HashMap<Uuid, ApprovalGate>,
    #[allow(dead_code)]
    timeout_check_interval_seconds: u64,
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self {
            gates: HashMap::new(),
            timeout_check_interval_seconds: 60,
        }
    }

    pub fn create_gate(
        &mut self,
        execution_id: Uuid,
        task_id: Uuid,
        agent_id: Uuid,
        action: AgentAction,
        reason: String,
    ) -> Uuid {
        let gate = ApprovalGate::new(execution_id, task_id, agent_id, action, reason);
        let id = gate.id;
        self.gates.insert(id, gate);
        id
    }

    pub fn get_gate(&self, id: Uuid) -> Option<&ApprovalGate> {
        self.gates.get(&id)
    }

    pub fn get_gate_mut(&mut self, id: Uuid) -> Option<&mut ApprovalGate> {
        self.gates.get_mut(&id)
    }

    pub fn approve_gate(
        &mut self,
        id: Uuid,
        resolved_by: &str,
        notes: Option<&str>,
    ) -> Result<(), ApprovalError> {
        let gate = self.gates.get_mut(&id).ok_or(ApprovalError::NotFound(id))?;

        if !gate.is_pending() {
            return Err(ApprovalError::AlreadyResolved(id));
        }

        gate.approve(resolved_by, notes);
        Ok(())
    }

    pub fn reject_gate(
        &mut self,
        id: Uuid,
        resolved_by: &str,
        notes: Option<&str>,
    ) -> Result<(), ApprovalError> {
        let gate = self.gates.get_mut(&id).ok_or(ApprovalError::NotFound(id))?;

        if !gate.is_pending() {
            return Err(ApprovalError::AlreadyResolved(id));
        }

        gate.reject(resolved_by, notes);
        Ok(())
    }

    pub fn cancel_gate(&mut self, id: Uuid) -> Result<(), ApprovalError> {
        let gate = self.gates.get_mut(&id).ok_or(ApprovalError::NotFound(id))?;

        gate.cancel();
        Ok(())
    }

    pub fn check_timeouts(&mut self) -> Vec<Uuid> {
        let mut timed_out = Vec::new();

        for (id, gate) in &mut self.gates {
            if gate.check_timeout() {
                timed_out.push(*id);
            }
        }

        timed_out
    }

    pub fn get_pending_gates(&self) -> Vec<&ApprovalGate> {
        self.gates.values().filter(|g| g.is_pending()).collect()
    }

    pub fn get_pending_for_task(&self, task_id: Uuid) -> Vec<&ApprovalGate> {
        self.gates
            .values()
            .filter(|g| g.task_id == task_id && g.is_pending())
            .collect()
    }

    pub fn get_pending_for_agent(&self, agent_id: Uuid) -> Vec<&ApprovalGate> {
        self.gates
            .values()
            .filter(|g| g.agent_id == agent_id && g.is_pending())
            .collect()
    }

    pub fn get_pending_for_execution(&self, execution_id: Uuid) -> Vec<&ApprovalGate> {
        self.gates
            .values()
            .filter(|g| g.execution_id == execution_id && g.is_pending())
            .collect()
    }

    pub fn remove_gate(&mut self, id: Uuid) -> Option<ApprovalGate> {
        self.gates.remove(&id)
    }

    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    pub fn pending_count(&self) -> usize {
        self.gates.values().filter(|g| g.is_pending()).count()
    }

    pub fn clear_resolved(&mut self) -> usize {
        let resolved_ids: Vec<Uuid> = self
            .gates
            .iter()
            .filter(|(_, g)| g.is_resolved())
            .map(|(id, _)| *id)
            .collect();

        let count = resolved_ids.len();
        for id in resolved_ids {
            self.gates.remove(&id);
        }
        count
    }
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApprovalError {
    #[error("approval gate not found: {0}")]
    NotFound(Uuid),
    #[error("approval gate already resolved: {0}")]
    AlreadyResolved(Uuid),
    #[error("approval gate timed out: {0}")]
    TimedOut(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_gate_creation() {
        let gate = ApprovalGate::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Need to delete file".to_string(),
        );

        assert!(gate.is_pending());
        assert!(!gate.is_resolved());
    }

    #[test]
    fn test_approval_gate_approve() {
        let mut gate = ApprovalGate::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test".to_string(),
        );

        gate.approve("user", Some("Looks good"));
        assert!(gate.is_approved());
        assert!(gate.is_resolved());
        assert_eq!(gate.resolved_by, Some("user".to_string()));
    }

    #[test]
    fn test_approval_gate_reject() {
        let mut gate = ApprovalGate::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test".to_string(),
        );

        gate.reject("user", Some("Not safe"));
        assert!(!gate.is_approved());
        assert!(gate.is_resolved());
        assert_eq!(gate.status, ApprovalStatus::Rejected);
    }

    #[test]
    fn test_approval_gate_cancel() {
        let mut gate = ApprovalGate::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test".to_string(),
        );

        gate.cancel();
        assert_eq!(gate.status, ApprovalStatus::Cancelled);
        assert!(gate.is_resolved());
    }

    #[test]
    fn test_approval_manager_creation() {
        let manager = ApprovalManager::new();
        assert_eq!(manager.gate_count(), 0);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_approval_manager_create_gate() {
        let mut manager = ApprovalManager::new();

        let id = manager.create_gate(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test".to_string(),
        );

        assert_eq!(manager.gate_count(), 1);
        assert_eq!(manager.pending_count(), 1);
        assert!(manager.get_gate(id).is_some());
    }

    #[test]
    fn test_approval_manager_approve() {
        let mut manager = ApprovalManager::new();

        let id = manager.create_gate(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test".to_string(),
        );

        manager.approve_gate(id, "user", Some("OK")).unwrap();
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_approval_manager_reject() {
        let mut manager = ApprovalManager::new();

        let id = manager.create_gate(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test".to_string(),
        );

        manager.reject_gate(id, "user", Some("No")).unwrap();
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_approval_manager_get_pending_for_task() {
        let mut manager = ApprovalManager::new();
        let task_id = Uuid::new_v4();

        manager.create_gate(
            Uuid::new_v4(),
            task_id,
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test 1".to_string(),
        );

        manager.create_gate(
            Uuid::new_v4(),
            task_id,
            Uuid::new_v4(),
            AgentAction::ModifyConfig,
            "Test 2".to_string(),
        );

        manager.create_gate(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test 3".to_string(),
        );

        let pending = manager.get_pending_for_task(task_id);
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_approval_manager_clear_resolved() {
        let mut manager = ApprovalManager::new();

        let id1 = manager.create_gate(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test 1".to_string(),
        );

        manager.create_gate(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test 2".to_string(),
        );

        manager.approve_gate(id1, "user", None).unwrap();

        let cleared = manager.clear_resolved();
        assert_eq!(cleared, 1);
        assert_eq!(manager.gate_count(), 1);
    }

    #[test]
    fn test_approval_gate_with_context() {
        let gate = ApprovalGate::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            AgentAction::DeleteFile,
            "Test".to_string(),
        )
        .with_context("file", "src/main.rs")
        .with_context("reason", "obsolete code");

        assert_eq!(gate.context.get("file"), Some(&"src/main.rs".to_string()));
        assert_eq!(
            gate.context.get("reason"),
            Some(&"obsolete code".to_string())
        );
    }
}
