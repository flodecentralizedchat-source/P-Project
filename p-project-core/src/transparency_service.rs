use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateCategory {
    Development,
    Treasury,
    Roadmap,
    Community,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub id: String,
    pub title: String,
    pub body: String,
    pub author: String,
    pub category: UpdateCategory,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub created_at: NaiveDateTime,
}

pub struct TransparencyService {
    updates: Vec<ProgressUpdate>,
}

impl TransparencyService {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    fn now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    pub fn record_update(
        &mut self,
        title: String,
        body: String,
        author: String,
        category: UpdateCategory,
        tags: Vec<String>,
        links: Vec<String>,
    ) -> String {
        let id = format!("upd_{}", uuid::Uuid::new_v4());
        let update = ProgressUpdate {
            id: id.clone(),
            title,
            body,
            author,
            category,
            tags,
            links,
            created_at: Self::now(),
        };
        self.updates.push(update);
        id
    }

    pub fn list_updates(
        &self,
        limit: Option<usize>,
        category: Option<UpdateCategory>,
    ) -> Vec<ProgressUpdate> {
        let mut items: Vec<_> = self
            .updates
            .iter()
            .filter(|u| category.as_ref().map_or(true, |c| &u.category == c))
            .cloned()
            .collect();
        // newest first
        items.sort_by_key(|u| std::cmp::Reverse(u.created_at));
        if let Some(lim) = limit {
            items.truncate(lim);
        }
        items
    }

    pub fn list_since(&self, since: NaiveDateTime) -> Vec<ProgressUpdate> {
        let mut items: Vec<_> = self
            .updates
            .iter()
            .filter(|u| u.created_at >= since)
            .cloned()
            .collect();
        items.sort_by_key(|u| std::cmp::Reverse(u.created_at));
        items
    }
}
