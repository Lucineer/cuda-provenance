/*!
# cuda-provenance

Decision lineage and data origin tracking.

Every action has a reason. Every piece of data has a source. Provenance
tracks *why* things happened so agents can explain, audit, and
reproduce their decisions.

- Decision records with reasoning chains
- Data lineage (source → transform → output)
- Accountability assignments
- Audit trails with tamper detection
- Provenance queries
*/

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Provenance record for a single decision
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub inputs: Vec<String>,        // data sources
    pub reasoning: String,
    pub confidence: f64,
    pub timestamp: u64,
    pub parent_id: Option<String>,  // caused by
    pub outcome: String,
    pub energy_cost: f64,
    pub tags: Vec<String>,
}

/// Data lineage entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineageEntry {
    pub data_id: String,
    pub source_type: LineageSource,
    pub source_id: String,
    pub transform: String,       // what was done to produce this
    pub derived_from: Vec<String>, // input data IDs
    pub timestamp: u64,
    pub agent_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageSource { Sensor, Agent, External, Computed, Stored }

/// Accountability entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountabilityEntry {
    pub action_id: String,
    pub responsible: String,      // agent or system
    pub role: AccountabilityRole,
    pub approved_by: Option<String>,
    pub timestamp: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountabilityRole { Actor, Reviewer, Approver, Delegator }

/// Audit event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: AuditEventType,
    pub actor: String,
    pub target: String,
    pub details: String,
    pub timestamp: u64,
    pub prev_hash: u64,  // simple chain integrity
    pub hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType { Decision, DataCreate, DataModify, DataDelete, PermissionChange, Error }

/// Provenance query result
#[derive(Clone, Debug)]
pub struct ProvenanceQuery {
    pub records: Vec<DecisionRecord>,
    pub lineage: Vec<LineageEntry>,
    pub accountability: Vec<AccountabilityEntry>,
}

/// The provenance tracker
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceTracker {
    pub decisions: HashMap<String, DecisionRecord>,
    pub lineage: HashMap<String, LineageEntry>,
    pub accountability: Vec<AccountabilityEntry>,
    pub audit_log: Vec<AuditEvent>,
    pub next_id: u64,
    pub chain_hash: u64,
}

impl ProvenanceTracker {
    pub fn new() -> Self { ProvenanceTracker { decisions: HashMap::new(), lineage: HashMap::new(), accountability: vec![], audit_log: vec![], next_id: 1, chain_hash: 0 } }

    fn new_id(&mut self) -> String { let id = format!("prov_{}", self.next_id); self.next_id += 1; id }

    /// Record a decision
    pub fn record_decision(&mut self, agent_id: &str, action: &str, inputs: &[&str], reasoning: &str, confidence: f64) -> String {
        let id = self.new_id();
        let record = DecisionRecord {
            id: id.clone(), agent_id: agent_id.to_string(), action: action.to_string(),
            inputs: inputs.iter().map(|s| s.to_string()).collect(),
            reasoning: reasoning.to_string(), confidence, timestamp: now(),
            parent_id: None, outcome: String::new(), energy_cost: 0.0, tags: vec![],
        };
        self.decisions.insert(id.clone(), record);
        self.audit(AuditEventType::Decision, agent_id, &id, &format!("{}: {}", action, reasoning));
        id
    }

    /// Link decision to parent (causality)
    pub fn link_cause(&mut self, child_id: &str, parent_id: &str) {
        if let Some(child) = self.decisions.get_mut(child_id) { child.parent_id = Some(parent_id.to_string()); }
    }

    /// Record data lineage
    pub fn record_lineage(&mut self, data_id: &str, source: LineageSource, source_id: &str, transform: &str, derived_from: &[&str], agent_id: &str) {
        let entry = LineageEntry {
            data_id: data_id.to_string(), source, source_id: source_id.to_string(),
            transform: transform.to_string(), derived_from: derived_from.iter().map(|s| s.to_string()).collect(),
            timestamp: now(), agent_id: agent_id.to_string(),
        };
        self.lineage.insert(data_id.to_string(), entry);
    }

    /// Record accountability
    pub fn record_accountability(&mut self, action_id: &str, responsible: &str, role: AccountabilityRole) {
        self.accountability.push(AccountabilityEntry { action_id: action_id.to_string(), responsible: responsible.to_string(), role, approved_by: None, timestamp: now() });
    }

    /// Audit trail
    fn audit(&mut self, event_type: AuditEventType, actor: &str, target: &str, details: &str) {
        let prev = self.chain_hash;
        let event_str = format!("{}:{}:{}:{}:{}", event_type as u8, actor, target, details, prev);
        let hash = simple_hash(&event_str);
        self.chain_hash = hash;
        self.audit_log.push(AuditEvent { id: self.new_id(), event_type, actor: actor.to_string(), target: target.to_string(), details: details.to_string(), timestamp: now(), prev_hash: prev, hash });
    }

    /// Query all decisions by agent
    pub fn decisions_by_agent(&self, agent_id: &str) -> Vec<&DecisionRecord> {
        self.decisions.values().filter(|d| d.agent_id == agent_id).collect()
    }

    /// Query decision chain (ancestors)
    pub fn decision_chain(&self, id: &str) -> Vec<&DecisionRecord> {
        let mut chain = vec![];
        let mut current = id;
        while let Some(record) = self.decisions.get(current) {
            chain.push(record);
            current = match &record.parent_id { Some(p) => p.as_str(), None => break };
        }
        chain
    }

    /// Query lineage for a data item
    pub fn lineage_of(&self, data_id: &str) -> Option<&LineageEntry> { self.lineage.get(data_id) }

    /// Verify audit chain integrity
    pub fn verify_chain(&self) -> bool {
        let mut expected_hash = 0u64;
        for event in &self.audit_log {
            if event.prev_hash != expected_hash { return false; }
            expected_hash = event.hash;
        }
        true
    }

    /// Summary
    pub fn summary(&self) -> String {
        format!("Provenance: {} decisions, {} lineage entries, {} accountability, {} audit events, chain_valid={}",
            self.decisions.len(), self.lineage.len(), self.accountability.len(), self.audit_log.len(), self.verify_chain())
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() { h = h.wrapping_mul(33).wrapping_add(b as u64); }
    h
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_decision() {
        let mut pt = ProvenanceTracker::new();
        let id = pt.record_decision("agent1", "navigate", &["sensor_1"], "path clear", 0.9);
        assert!(pt.decisions.contains_key(&id));
    }

    #[test]
    fn test_cause_linking() {
        let mut pt = ProvenanceTracker::new();
        let parent = pt.record_decision("agent1", "observe", &[], "look around", 0.95);
        let child = pt.record_decision("agent1", "navigate", &[], "go there", 0.85);
        pt.link_cause(&child, &parent);
        let chain = pt.decision_chain(&child);
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_lineage() {
        let mut pt = ProvenanceTracker::new();
        pt.record_lineage("output_1", LineageSource::Computed, "agent1", "sum(a,b)", &["input_a", "input_b"], "agent1");
        let entry = pt.lineage_of("output_1").unwrap();
        assert_eq!(entry.derived_from, vec!["input_a", "input_b"]);
    }

    #[test]
    fn test_accountability() {
        let mut pt = ProvenanceTracker::new();
        pt.record_accountability("action_1", "agent1", AccountabilityRole::Actor);
        assert_eq!(pt.accountability.len(), 1);
    }

    #[test]
    fn test_decisions_by_agent() {
        let mut pt = ProvenanceTracker::new();
        pt.record_decision("a1", "x", &[], "", 0.5);
        pt.record_decision("a2", "y", &[], "", 0.5);
        pt.record_decision("a1", "z", &[], "", 0.5);
        assert_eq!(pt.decisions_by_agent("a1").len(), 2);
    }

    #[test]
    fn test_audit_chain_valid() {
        let mut pt = ProvenanceTracker::new();
        pt.record_decision("a1", "x", &[], "reason", 0.9);
        pt.record_decision("a2", "y", &[], "reason", 0.8);
        assert!(pt.verify_chain());
    }

    #[test]
    fn test_audit_events_created() {
        let mut pt = ProvenanceTracker::new();
        pt.record_decision("a1", "x", &[], "", 0.9);
        assert_eq!(pt.audit_log.len(), 1);
        assert_eq!(pt.audit_log[0].event_type, AuditEventType::Decision);
    }

    #[test]
    fn test_data_sources() {
        let mut pt = ProvenanceTracker::new();
        pt.record_lineage("s1", LineageSource::Sensor, "temp_sensor", "read", &[], "agent1");
        pt.record_lineage("s2", LineageSource::External, "api", "fetch", &[], "agent1");
        pt.record_lineage("s3", LineageSource::Computed, "agent1", "compute", &["s1", "s2"], "agent1");
        assert_eq!(pt.lineage.len(), 3);
    }

    #[test]
    fn test_decision_fields() {
        let mut pt = ProvenanceTracker::new();
        let id = pt.record_decision("agent1", "scan", &["lidar", "camera"], "obstacle detected", 0.92);
        let d = &pt.decisions[&id];
        assert_eq!(d.inputs.len(), 2);
        assert!((d.confidence - 0.92).abs() < 0.001);
    }

    #[test]
    fn test_summary() {
        let pt = ProvenanceTracker::new();
        let s = pt.summary();
        assert!(s.contains("chain_valid=true"));
    }
}
