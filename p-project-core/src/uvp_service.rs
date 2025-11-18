use crate::models::UniqueValueProposition;

pub struct UvpEngine {
    uvps: Vec<UniqueValueProposition>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UvpSummaryItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub metric_key: String,
    pub multiplier: f64,
}

impl UvpEngine {
    pub fn new(initial: Vec<UniqueValueProposition>) -> Self {
        Self { uvps: initial }
    }

    pub fn add_uvp(&mut self, uvp: UniqueValueProposition) {
        self.uvps.push(uvp);
    }

    pub fn list(&self) -> &[UniqueValueProposition] {
        &self.uvps
    }

    pub fn summary(&self) -> Vec<UvpSummaryItem> {
        self.uvps
            .iter()
            .map(|u| UvpSummaryItem {
                id: u.id.clone(),
                name: u.name.clone(),
                description: u.description.clone(),
                metric_key: u.metric_key.clone(),
                multiplier: u.multiplier,
            })
            .collect()
    }

    // Compute combined multiplier based on presence/positivity of each metric
    pub fn compute_reward_multiplier(&self, ctx: &serde_json::Value) -> f64 {
        let mut m = 1.0f64;
        for u in &self.uvps {
            if let Some(v) = ctx.get(&u.metric_key) {
                let positive = match v {
                    serde_json::Value::Bool(b) => *b,
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) > 0.0,
                    serde_json::Value::String(s) => !s.is_empty(),
                    _ => false,
                };
                if positive {
                    m *= u.multiplier;
                }
            }
        }
        m
    }
}
