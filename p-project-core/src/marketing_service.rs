//! Marketing and Community Building service for P-Project
//!
//! Covers three areas:
//! - Content Marketing (create, publish, list)
//! - Social Media Scheduling (schedule, post, list)
//! - Influencer Partnerships (register, search, offer lifecycle)

use chrono::{NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    Article,
    Thread,
    Guide,
    Video,
    Newsletter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentStatus {
    Draft,
    Scheduled,
    Published,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketingContent {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub content_type: ContentType,
    pub tags: Vec<String>,
    pub seo_keywords: Vec<String>,
    pub status: ContentStatus,
    pub created_at: NaiveDateTime,
    pub published_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SocialPlatform {
    Twitter,
    Telegram,
    Discord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PostStatus {
    Scheduled,
    Posted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPost {
    pub id: String,
    pub content_id: Option<String>,
    pub platform: SocialPlatform,
    pub text: String,
    pub scheduled_at: NaiveDateTime,
    pub posted_at: Option<NaiveDateTime>,
    pub status: PostStatus,
    pub post_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Influencer {
    pub id: String,
    pub name: String,
    pub platform: SocialPlatform,
    pub handle: String,
    pub reach: u64,
    pub alignment_tags: Vec<String>,
    pub alignment_score: f32, // 0.0 - 1.0
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OfferStatus {
    Proposed,
    Accepted,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipOffer {
    pub id: String,
    pub influencer_id: String,
    pub content_id: Option<String>,
    pub compensation_tokens: Decimal,
    pub deliverables: Vec<String>,
    pub status: OfferStatus,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Default)]
pub struct MarketingService {
    contents: HashMap<String, MarketingContent>,
    posts: HashMap<String, SocialPost>,
    influencers: HashMap<String, Influencer>,
    offers: HashMap<String, PartnershipOffer>,
}

impl MarketingService {
    pub fn new() -> Self {
        Self::default()
    }

    // ---------- Content Marketing ----------
    pub fn create_content(
        &mut self,
        title: &str,
        summary: &str,
        body: &str,
        content_type: ContentType,
        tags: Vec<String>,
        seo_keywords: Vec<String>,
    ) -> MarketingContent {
        let id = crate::utils::generate_id();
        let content = MarketingContent {
            id: id.clone(),
            title: title.to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            content_type,
            tags,
            seo_keywords,
            status: ContentStatus::Draft,
            created_at: Utc::now().naive_utc(),
            published_at: None,
        };
        self.contents.insert(id.clone(), content.clone());
        content
    }

    pub fn publish_content(&mut self, content_id: &str) -> Result<MarketingContent, String> {
        let now = Utc::now().naive_utc();
        let entry = self
            .contents
            .get_mut(content_id)
            .ok_or_else(|| "content_not_found".to_string())?;
        entry.status = ContentStatus::Published;
        entry.published_at = Some(now);
        Ok(entry.clone())
    }

    pub fn list_content(&self, limit: usize) -> Vec<MarketingContent> {
        let mut v: Vec<_> = self.contents.values().cloned().collect();
        v.sort_by_key(|c| c.created_at);
        v.into_iter().rev().take(limit.max(1)).collect()
    }

    // ---------- Social Media Scheduling ----------
    pub fn schedule_social_post(
        &mut self,
        content_id: Option<String>,
        platform: SocialPlatform,
        text: &str,
        scheduled_at: NaiveDateTime,
    ) -> Result<SocialPost, String> {
        if let Some(ref id) = content_id {
            if !self.contents.contains_key(id) {
                return Err("linked_content_not_found".to_string());
            }
        }

        let id = crate::utils::generate_id();
        let post = SocialPost {
            id: id.clone(),
            content_id,
            platform,
            text: text.to_string(),
            scheduled_at,
            posted_at: None,
            status: PostStatus::Scheduled,
            post_url: None,
        };
        self.posts.insert(id.clone(), post.clone());
        Ok(post)
    }

    pub fn mark_posted(&mut self, post_id: &str, post_url: &str) -> Result<SocialPost, String> {
        let now = Utc::now().naive_utc();
        let post = self
            .posts
            .get_mut(post_id)
            .ok_or_else(|| "post_not_found".to_string())?;
        post.status = PostStatus::Posted;
        post.posted_at = Some(now);
        post.post_url = Some(post_url.to_string());
        Ok(post.clone())
    }

    pub fn list_scheduled_posts(&self, limit: usize) -> Vec<SocialPost> {
        let mut v: Vec<_> = self
            .posts
            .values()
            .filter(|p| p.status == PostStatus::Scheduled)
            .cloned()
            .collect();
        v.sort_by_key(|p| p.scheduled_at);
        v.into_iter().take(limit.max(1)).collect()
    }

    // ---------- Influencer Partnerships ----------
    pub fn register_influencer(
        &mut self,
        name: &str,
        platform: SocialPlatform,
        handle: &str,
        reach: u64,
        alignment_tags: Vec<String>,
        alignment_score: f32,
    ) -> Influencer {
        let id = crate::utils::generate_id();
        let influencer = Influencer {
            id: id.clone(),
            name: name.to_string(),
            platform,
            handle: handle.to_string(),
            reach,
            alignment_tags,
            alignment_score: alignment_score.clamp(0.0, 1.0),
            created_at: Utc::now().naive_utc(),
        };
        self.influencers.insert(id.clone(), influencer.clone());
        influencer
    }

    pub fn search_influencers(&self, min_reach: u64, required_tags: &[String]) -> Vec<Influencer> {
        let mut v: Vec<_> = self
            .influencers
            .values()
            .filter(|i| i.reach >= min_reach)
            .filter(|i| {
                required_tags
                    .iter()
                    .all(|t| i.alignment_tags.iter().any(|it| it.eq_ignore_ascii_case(t)))
            })
            .cloned()
            .collect();
        v.sort_by(|a, b| b.reach.cmp(&a.reach));
        v
    }

    pub fn create_partnership_offer(
        &mut self,
        influencer_id: &str,
        content_id: Option<String>,
        compensation_tokens: Decimal,
        deliverables: Vec<String>,
        notes: Option<String>,
    ) -> Result<PartnershipOffer, String> {
        if !self.influencers.contains_key(influencer_id) {
            return Err("influencer_not_found".to_string());
        }
        if let Some(ref c) = content_id {
            if !self.contents.contains_key(c) {
                return Err("content_not_found".to_string());
            }
        }
        if compensation_tokens < Decimal::ZERO {
            return Err("invalid_compensation".to_string());
        }
        let id = crate::utils::generate_id();
        let now = Utc::now().naive_utc();
        let offer = PartnershipOffer {
            id: id.clone(),
            influencer_id: influencer_id.to_string(),
            content_id,
            compensation_tokens,
            deliverables,
            status: OfferStatus::Proposed,
            notes,
            created_at: now,
            updated_at: now,
        };
        self.offers.insert(id.clone(), offer.clone());
        Ok(offer)
    }

    pub fn respond_to_offer(
        &mut self,
        offer_id: &str,
        accept: bool,
    ) -> Result<PartnershipOffer, String> {
        let now = Utc::now().naive_utc();
        let offer = self
            .offers
            .get_mut(offer_id)
            .ok_or_else(|| "offer_not_found".to_string())?;
        offer.status = if accept {
            OfferStatus::Accepted
        } else {
            OfferStatus::Rejected
        };
        offer.updated_at = now;
        Ok(offer.clone())
    }

    pub fn list_offers_for_influencer(&self, influencer_id: &str) -> Vec<PartnershipOffer> {
        let mut v: Vec<_> = self
            .offers
            .values()
            .filter(|o| o.influencer_id == influencer_id)
            .cloned()
            .collect();
        v.sort_by_key(|o| o.created_at);
        v.into_iter().rev().collect()
    }
}
