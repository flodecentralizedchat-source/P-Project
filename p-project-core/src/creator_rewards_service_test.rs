use super::creator_rewards_service::*;

fn default_config() -> CreatorRewardsConfig {
    CreatorRewardsConfig {
        base_reward_blog: 10.0,
        min_blog_word_count: 300,
        base_reward_youtube: 20.0,
        min_video_duration_seconds: 60,
        vote_reward_per_vote: 1.0,
        vote_daily_cap_per_content: 3.0,
        max_reward_per_content: 50.0,
        peace_keywords: vec!["peace".to_string(), "harmony".to_string()],
    }
}

#[test]
fn blogger_earns_for_peace_article() {
    let mut service = CreatorRewardsService::new(default_config());
    let creator = service
        .register_creator(CreateCreatorRequest {
            name: "Alice".into(),
            creator_type: CreatorType::Blogger,
            wallet_address: "P_wallet_1".into(),
            bio: None,
            social_links: None,
        })
        .unwrap();

    // Peace article with sufficient word count
    let resp = service
        .submit_content(SubmitContentRequest {
            creator_id: creator.id.clone(),
            content_type: ContentType::BlogArticle,
            title: "Peace in our time".into(),
            url: Some("https://example.com/peace".into()),
            tags: vec!["community".into(), "peace".into()],
            word_count: Some(500),
            duration_seconds: None,
        })
        .unwrap();
    assert!(resp.content.is_peace_content);
    assert_eq!(resp.reward_credited, 10.0);
    assert_eq!(service.get_creator_balance(&creator.id), 10.0);

    // Non-peace article doesn't earn
    let resp2 = service
        .submit_content(SubmitContentRequest {
            creator_id: creator.id.clone(),
            content_type: ContentType::BlogArticle,
            title: "My routine".into(),
            url: None,
            tags: vec!["daily".into()],
            word_count: Some(600),
            duration_seconds: None,
        })
        .unwrap();
    assert!(!resp2.content.is_peace_content);
    assert_eq!(resp2.reward_credited, 0.0);
    assert_eq!(service.get_creator_balance(&creator.id), 10.0);
}

#[test]
fn youtuber_earns_for_peace_content() {
    let mut service = CreatorRewardsService::new(default_config());
    let creator = service
        .register_creator(CreateCreatorRequest {
            name: "Bob".into(),
            creator_type: CreatorType::YouTuber,
            wallet_address: "P_wallet_2".into(),
            bio: None,
            social_links: None,
        })
        .unwrap();

    let resp = service
        .submit_content(SubmitContentRequest {
            creator_id: creator.id.clone(),
            content_type: ContentType::YouTubeVideo,
            title: "Harmony through Dialogue".into(),
            url: Some("https://youtube.com/watch?v=xyz".into()),
            tags: vec!["peace".into(), "dialogue".into()],
            word_count: None,
            duration_seconds: Some(120),
        })
        .unwrap();
    assert!(resp.content.is_peace_content);
    assert_eq!(resp.reward_credited, 20.0);
    assert_eq!(service.get_creator_balance(&creator.id), 20.0);

    // Too short video or non-peace shouldn't earn
    let resp2 = service
        .submit_content(SubmitContentRequest {
            creator_id: creator.id.clone(),
            content_type: ContentType::YouTubeVideo,
            title: "Quick update".into(),
            url: None,
            tags: vec!["update".into()],
            word_count: None,
            duration_seconds: Some(30),
        })
        .unwrap();
    assert!(!resp2.content.is_peace_content);
    assert_eq!(resp2.reward_credited, 0.0);
    assert_eq!(service.get_creator_balance(&creator.id), 20.0);
}

#[test]
fn podcaster_earns_for_community_votes_with_caps() {
    let mut cfg = default_config();
    cfg.vote_reward_per_vote = 1.0;
    cfg.vote_daily_cap_per_content = 3.0; // at most 3 P-Coin per day per content
    cfg.max_reward_per_content = 5.0; // total per content cap
    let mut service = CreatorRewardsService::new(cfg);

    let creator = service
        .register_creator(CreateCreatorRequest {
            name: "Carol".into(),
            creator_type: CreatorType::Podcaster,
            wallet_address: "P_wallet_3".into(),
            bio: None,
            social_links: None,
        })
        .unwrap();

    let resp = service
        .submit_content(SubmitContentRequest {
            creator_id: creator.id.clone(),
            content_type: ContentType::PodcastEpisode,
            title: "Community Peace Voices".into(),
            url: Some("https://podcasts.example/peace".into()),
            tags: vec!["community".into(), "peace".into()],
            word_count: None,
            duration_seconds: Some(1800),
        })
        .unwrap();
    let content_id = resp.content.id.clone();
    assert_eq!(resp.reward_credited, 0.0);
    assert_eq!(service.get_creator_balance(&creator.id), 0.0);

    // 5 unique votes; daily cap 3 → only first 3 count
    for i in 0..5 {
        let amt = service
            .record_vote(&content_id, &format!("voter_{}", i))
            .unwrap();
        if i < 3 {
            assert_eq!(amt, 1.0);
        } else {
            assert_eq!(amt, 0.0);
        }
    }
    assert_eq!(service.get_creator_balance(&creator.id), 3.0);

    // Additional votes until hitting total cap 5.0
    for i in 5..10 {
        let _ = service
            .record_vote(&content_id, &format!("voter_{}", i))
            .unwrap_or(0.0);
    }
    // Balance should not exceed total per-content cap (5.0)
    assert_eq!(service.get_creator_balance(&creator.id), 5.0);

    // Duplicate vote should error
    let dup = service.record_vote(&content_id, "voter_0");
    assert!(dup.is_err());
}

#[test]
fn type_mismatch_is_rejected() {
    let mut service = CreatorRewardsService::new(default_config());
    let blogger = service
        .register_creator(CreateCreatorRequest {
            name: "Dave".into(),
            creator_type: CreatorType::Blogger,
            wallet_address: "P_wallet_4".into(),
            bio: None,
            social_links: None,
        })
        .unwrap();

    let err = service
        .submit_content(SubmitContentRequest {
            creator_id: blogger.id.clone(),
            content_type: ContentType::YouTubeVideo,
            title: "Wrong type".into(),
            url: None,
            tags: vec![],
            word_count: None,
            duration_seconds: None,
        })
        .unwrap_err();
    assert!(err.contains("not allowed"));
}

#[test]
fn withdrawal_reduces_balance() {
    let mut service = CreatorRewardsService::new(default_config());
    let creator = service
        .register_creator(CreateCreatorRequest {
            name: "Eve".into(),
            creator_type: CreatorType::Blogger,
            wallet_address: "P_wallet_5".into(),
            bio: None,
            social_links: None,
        })
        .unwrap();

    let _ = service
        .submit_content(SubmitContentRequest {
            creator_id: creator.id.clone(),
            content_type: ContentType::BlogArticle,
            title: "Peaceful habits".into(),
            url: None,
            tags: vec!["peace".into()],
            word_count: Some(350),
            duration_seconds: None,
        })
        .unwrap();
    assert_eq!(service.get_creator_balance(&creator.id), 10.0);

    let tx = service.withdraw_rewards(&creator.id, 6.0).unwrap();
    assert_eq!(tx.amount, 6.0);
    assert_eq!(service.get_creator_balance(&creator.id), 4.0);

    let err = service.withdraw_rewards(&creator.id, 10.0).unwrap_err();
    assert!(err.contains("insufficient"));
}
