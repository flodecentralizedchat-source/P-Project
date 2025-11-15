//! Creator rewards service for content-based incentives (blogs, videos, podcasts)
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Types of creators supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreatorType {
    Blogger,
    YouTuber,
    Podcaster,
}

/// Types of content submissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    BlogArticle,
    YouTubeVideo,
    PodcastEpisode,
}

/// Creator profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorProfile {
    pub id: String,
    pub name: String,
    pub creator_type: CreatorType,
    pub wallet_address: String,
    pub bio: Option<String>,
    pub social_links: Option<HashMap<String, String>>, // platform -> url
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub total_rewards: f64,
    pub content_count: usize,
}

/// Content submission record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSubmission {
    pub id: String,
    pub creator_id: String,
    pub content_type: ContentType,
    pub title: String,
    pub url: Option<String>,
    pub tags: Vec<String>,
    pub word_count: Option<u32>,
    pub duration_seconds: Option<u32>,
    pub created_at: NaiveDateTime,
    pub is_peace_content: bool,
    pub reward_issued: f64,
    pub votes: HashSet<String>, // unique voter ids
}

/// Reward transaction for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardTransaction {
    pub id: String,
    pub creator_id: String,
    pub content_id: Option<String>,
    pub amount: f64,
    pub reason: String,
    pub timestamp: NaiveDateTime,
}

/// Configuration parameters for rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorRewardsConfig {
    pub base_reward_blog: f64,
    pub min_blog_word_count: u32,
    pub base_reward_youtube: f64,
    pub min_video_duration_seconds: u32,
    pub vote_reward_per_vote: f64,
    pub vote_daily_cap_per_content: f64,
    pub max_reward_per_content: f64,
    pub peace_keywords: Vec<String>,
}

impl Default for CreatorRewardsConfig {
    fn default() -> Self {
        Self {
            base_reward_blog: 10.0,
            min_blog_word_count: 300,
            base_reward_youtube: 20.0,
            min_video_duration_seconds: 60,
            vote_reward_per_vote: 1.0,
            vote_daily_cap_per_content: 10.0,
            max_reward_per_content: 100.0,
            peace_keywords: vec![
                "peace".to_string(),
                "nonviolent".to_string(),
                "harmony".to_string(),
            ],
        }
    }
}

/// Request to register a new creator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreatorRequest {
    pub name: String,
    pub creator_type: CreatorType,
    pub wallet_address: String,
    pub bio: Option<String>,
    pub social_links: Option<HashMap<String, String>>, // platform -> url
}

/// Request to submit content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitContentRequest {
    pub creator_id: String,
    pub content_type: ContentType,
    pub title: String,
    pub url: Option<String>,
    pub tags: Vec<String>,
    pub word_count: Option<u32>,
    pub duration_seconds: Option<u32>,
}

/// Response for a content submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitContentResponse {
    pub content: ContentSubmission,
    pub reward_credited: f64,
}

/// Service implementing creator rewards
pub struct CreatorRewardsService {
    pub config: CreatorRewardsConfig,
    pub creators: HashMap<String, CreatorProfile>,
    pub contents: HashMap<String, ContentSubmission>,
    pub content_index_by_creator: HashMap<String, Vec<String>>, // creator_id -> [content_id]
    pub balances: HashMap<String, f64>,                         // creator_id -> pending rewards
    pub ledger: Vec<RewardTransaction>,
    // Track daily vote rewards per content to enforce daily caps
    vote_reward_today: HashMap<String, (NaiveDate, f64)>, // content_id -> (date, amount_today)
}

impl CreatorRewardsService {
    pub fn new(config: CreatorRewardsConfig) -> Self {
        Self {
            config,
            creators: HashMap::new(),
            contents: HashMap::new(),
            content_index_by_creator: HashMap::new(),
            balances: HashMap::new(),
            ledger: Vec::new(),
            vote_reward_today: HashMap::new(),
        }
    }

    fn now(&self) -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn gen_id(prefix: &str) -> String {
        format!("{}_{}", prefix, uuid::Uuid::new_v4())
    }

    fn ensure_creator(&self, creator_id: &str) -> Result<&CreatorProfile, String> {
        self.creators
            .get(creator_id)
            .ok_or_else(|| "creator not found".to_string())
    }

    fn allowed_content_for(creator_type: &CreatorType, content_type: &ContentType) -> bool {
        matches!(
            (creator_type, content_type),
            (CreatorType::Blogger, ContentType::BlogArticle)
                | (CreatorType::YouTuber, ContentType::YouTubeVideo)
                | (CreatorType::Podcaster, ContentType::PodcastEpisode)
        )
    }

    fn is_peace_content(&self, title: &str, tags: &[String]) -> bool {
        let title_lc = title.to_lowercase();
        let tag_lc: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
        self.config.peace_keywords.iter().any(|kw| {
            let kw_lc = kw.to_lowercase();
            title_lc.contains(&kw_lc) || tag_lc.iter().any(|t| t.contains(&kw_lc))
        })
    }

    fn credit_reward(
        &mut self,
        creator_id: &str,
        content_id: Option<String>,
        amount: f64,
        reason: &str,
    ) {
        if amount <= 0.0 {
            return;
        }
        let entry = self.balances.entry(creator_id.to_string()).or_insert(0.0);
        *entry += amount;
        if let Some(c) = self.creators.get_mut(creator_id) {
            c.total_rewards += amount;
        }
        self.ledger.push(RewardTransaction {
            id: Self::gen_id("reward"),
            creator_id: creator_id.to_string(),
            content_id,
            amount,
            reason: reason.to_string(),
            timestamp: self.now(),
        });
    }

    pub fn register_creator(
        &mut self,
        req: CreateCreatorRequest,
    ) -> Result<CreatorProfile, String> {
        if req.name.trim().is_empty() {
            return Err("name is required".to_string());
        }
        if req.wallet_address.trim().is_empty() {
            return Err("wallet address is required".to_string());
        }
        let id = Self::gen_id("creator");
        let profile = CreatorProfile {
            id: id.clone(),
            name: req.name,
            creator_type: req.creator_type,
            wallet_address: req.wallet_address,
            bio: req.bio,
            social_links: req.social_links,
            is_verified: true,
            created_at: self.now(),
            total_rewards: 0.0,
            content_count: 0,
        };
        self.creators.insert(id.clone(), profile.clone());
        Ok(profile)
    }

    pub fn submit_content(
        &mut self,
        req: SubmitContentRequest,
    ) -> Result<SubmitContentResponse, String> {
        let creator = self.ensure_creator(&req.creator_id)?.clone();
        if !Self::allowed_content_for(&creator.creator_type, &req.content_type) {
            return Err("content type not allowed for creator type".to_string());
        }

        let is_peace = self.is_peace_content(&req.title, &req.tags);
        let mut reward = 0.0;
        match req.content_type {
            ContentType::BlogArticle => {
                let wc = req.word_count.unwrap_or(0);
                if is_peace && wc >= self.config.min_blog_word_count {
                    reward = self.config.base_reward_blog;
                }
            }
            ContentType::YouTubeVideo => {
                let dur = req.duration_seconds.unwrap_or(0);
                if is_peace && dur >= self.config.min_video_duration_seconds {
                    reward = self.config.base_reward_youtube;
                }
            }
            ContentType::PodcastEpisode => {
                // Base reward is zero; rewards come from community votes
                reward = 0.0;
            }
        }

        let id = Self::gen_id("content");
        let mut content = ContentSubmission {
            id: id.clone(),
            creator_id: req.creator_id.clone(),
            content_type: req.content_type,
            title: req.title,
            url: req.url,
            tags: req.tags,
            word_count: req.word_count,
            duration_seconds: req.duration_seconds,
            created_at: self.now(),
            is_peace_content: is_peace,
            reward_issued: 0.0,
            votes: HashSet::new(),
        };

        // Apply cap per content
        if reward > 0.0 {
            let capped = reward.min(self.config.max_reward_per_content);
            content.reward_issued += capped;
            self.credit_reward(
                &content.creator_id,
                Some(id.clone()),
                capped,
                "content_submission",
            );
        }

        // Update indices
        self.contents.insert(id.clone(), content.clone());
        self.content_index_by_creator
            .entry(creator.id)
            .or_default()
            .push(id.clone());

        if let Some(c) = self.creators.get_mut(&req.creator_id) {
            c.content_count += 1;
        }

        Ok(SubmitContentResponse {
            content,
            reward_credited: reward.min(self.config.max_reward_per_content),
        })
    }

    /// Record a community vote for a content item (primarily for podcasts)
    /// Returns the reward credited for this vote (could be 0 if capped or duplicate)
    pub fn record_vote(&mut self, content_id: &str, voter_id: &str) -> Result<f64, String> {
        let today = Utc::now().naive_utc().date();
        let reward_info = {
            let content = self
                .contents
                .get_mut(content_id)
                .ok_or_else(|| "content not found".to_string())?;

            if content.votes.contains(voter_id) {
                return Err("duplicate vote".to_string());
            }

            // Only podcasters earn from votes; others ignore but allow dedup logic
            if content.content_type != ContentType::PodcastEpisode {
                // Allow voting for transparency but no reward
                content.votes.insert(voter_id.to_string());
                return Ok(0.0);
            }

            // Initialize/reset daily tracker
            let (ref mut tracked_date, ref mut amount_today) = self
                .vote_reward_today
                .entry(content_id.to_string())
                .or_insert((today, 0.0));
            if *tracked_date != today {
                *tracked_date = today;
                *amount_today = 0.0;
            }

            // Compute how much we can credit, respecting daily and per-content caps
            let remaining_daily = (self.config.vote_daily_cap_per_content - *amount_today).max(0.0);
            let remaining_total =
                (self.config.max_reward_per_content - content.reward_issued).max(0.0);
            let creditable = self
                .config
                .vote_reward_per_vote
                .min(remaining_daily)
                .min(remaining_total);

            // Record vote and possibly credit
            content.votes.insert(voter_id.to_string());
            if creditable > 0.0 {
                *amount_today += creditable;
                content.reward_issued += creditable;
                Some((content.creator_id.clone(), content.id.clone(), creditable))
            } else {
                None
            }
        };

        if let Some((creator_id, content_id, amount)) = reward_info {
            self.credit_reward(&creator_id, Some(content_id), amount, "community_vote");
            Ok(amount)
        } else {
            Ok(0.0)
        }
    }

    pub fn get_creator_balance(&self, creator_id: &str) -> f64 {
        *self.balances.get(creator_id).unwrap_or(&0.0)
    }

    pub fn withdraw_rewards(
        &mut self,
        creator_id: &str,
        amount: f64,
    ) -> Result<RewardTransaction, String> {
        if amount <= 0.0 {
            return Err("amount must be positive".to_string());
        }
        let bal = self.balances.entry(creator_id.to_string()).or_insert(0.0);
        if *bal < amount {
            return Err("insufficient balance".to_string());
        }
        *bal -= amount;
        let tx = RewardTransaction {
            id: Self::gen_id("withdrawal"),
            creator_id: creator_id.to_string(),
            content_id: None,
            amount,
            reason: "withdrawal".to_string(),
            timestamp: self.now(),
        };
        self.ledger.push(tx.clone());
        Ok(tx)
    }

    pub fn list_content_by_creator(&self, creator_id: &str) -> Vec<ContentSubmission> {
        self.content_index_by_creator
            .get(creator_id)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|id| self.contents.get(id))
            .cloned()
            .collect()
    }
}
