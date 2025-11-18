use crate::models::{ComponentStatus, EcosystemComponent, EcosystemLink};
use std::collections::HashMap;

pub struct EcosystemGraph {
    components: HashMap<String, EcosystemComponent>,
    links: Vec<EcosystemLink>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthSummary {
    pub healthy: usize,
    pub degraded: usize,
    pub down: usize,
    pub total_components: usize,
    pub total_links: usize,
}

impl EcosystemGraph {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            links: Vec::new(),
        }
    }

    pub fn add_component(&mut self, c: EcosystemComponent) {
        self.components.insert(c.id.clone(), c);
    }

    pub fn add_link(&mut self, l: EcosystemLink) -> Result<(), String> {
        if !self.components.contains_key(&l.from_id) {
            return Err(format!("missing from_id component: {}", l.from_id));
        }
        if !self.components.contains_key(&l.to_id) {
            return Err(format!("missing to_id component: {}", l.to_id));
        }
        self.links.push(l);
        Ok(())
    }

    pub fn get_component(&self, id: &str) -> Option<&EcosystemComponent> {
        self.components.get(id)
    }

    pub fn list_components(&self) -> Vec<&EcosystemComponent> {
        self.components.values().collect()
    }

    pub fn list_links(&self) -> &Vec<EcosystemLink> {
        &self.links
    }

    pub fn dependencies_for(&self, id: &str) -> Vec<&EcosystemComponent> {
        let mut out = Vec::new();
        for l in &self.links {
            if l.to_id == id {
                if let Some(c) = self.components.get(&l.from_id) {
                    out.push(c);
                }
            }
        }
        out
    }

    pub fn health_summary(&self) -> HealthSummary {
        let mut healthy = 0usize;
        let mut degraded = 0usize;
        let mut down = 0usize;
        for c in self.components.values() {
            match c.status {
                ComponentStatus::Healthy => healthy += 1,
                ComponentStatus::Degraded => degraded += 1,
                ComponentStatus::Down => down += 1,
            }
        }
        HealthSummary {
            healthy,
            degraded,
            down,
            total_components: self.components.len(),
            total_links: self.links.len(),
        }
    }
}
