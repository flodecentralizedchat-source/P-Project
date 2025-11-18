#[cfg(test)]
mod tests {
    use super::super::marketing_service::*;
    use rust_decimal::Decimal;

    #[test]
    fn content_create_publish_and_list() {
        let mut svc = MarketingService::new();
        let c = svc.create_content(
            "Launch Plan",
            "Overview of DEX listing",
            "Long form body",
            ContentType::Guide,
            vec!["dex".into(), "launch".into()],
            vec!["p-coin".into(), "listing".into()],
        );
        assert_eq!(c.status, ContentStatus::Draft);

        let p = svc.publish_content(&c.id).expect("publish");
        assert_eq!(p.status, ContentStatus::Published);
        assert!(p.published_at.is_some());

        let listed = svc.list_content(10);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title, "Launch Plan");
    }

    #[test]
    fn social_schedule_and_mark_posted() {
        let mut svc = MarketingService::new();
        let c = svc.create_content(
            "Thread",
            "Short",
            "Body",
            ContentType::Thread,
            vec![],
            vec![],
        );
        let when = chrono::Utc::now().naive_utc();
        let post = svc
            .schedule_social_post(Some(c.id.clone()), SocialPlatform::Twitter, "Hello X", when)
            .expect("schedule");
        assert_eq!(post.status, PostStatus::Scheduled);

        let scheduled = svc.list_scheduled_posts(5);
        assert_eq!(scheduled.len(), 1);

        let posted = svc
            .mark_posted(&post.id, "https://x.com/post/123")
            .expect("mark posted");
        assert_eq!(posted.status, PostStatus::Posted);
        assert!(posted.posted_at.is_some());
        assert!(posted.post_url.as_deref().unwrap().contains("x.com"));
    }

    #[test]
    fn influencer_register_search_and_offers() {
        let mut svc = MarketingService::new();
        let inf = svc.register_influencer(
            "Alice",
            SocialPlatform::Telegram,
            "@alice",
            150_000,
            vec!["crypto".into(), "education".into()],
            0.9,
        );
        let matches = svc.search_influencers(100_000, &vec!["crypto".into()]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, inf.id);

        let content = svc.create_content(
            "Explainer",
            "Why P-COIN",
            "Body",
            ContentType::Article,
            vec![],
            vec![],
        );
        let offer = svc
            .create_partnership_offer(
                &inf.id,
                Some(content.id.clone()),
                Decimal::from(10_000u64),
                vec!["1 post".into(), "1 AMA".into()],
                Some("Launch week".into()),
            )
            .expect("offer");
        assert_eq!(offer.status, OfferStatus::Proposed);

        let accepted = svc.respond_to_offer(&offer.id, true).expect("accept offer");
        assert_eq!(accepted.status, OfferStatus::Accepted);

        let list = svc.list_offers_for_influencer(&inf.id);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, offer.id);
    }
}
