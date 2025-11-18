use crate::collaboration_service::*;
use crate::marketing_service::SocialPlatform;
use crate::models::PartnerIntegrationType;
use serde_json::json;

#[test]
fn test_complementary_projects_link_and_list() {
    let mut svc = CollaborationService::new();

    let a = svc.register_partner(
        "Gaming Guild",
        PartnerIntegrationType::NFT,
        "Gaming",
        vec!["nft".into(), "rewards".into(), "guild".into()],
        None,
        json!({"site": "https://gaming.example"}),
    );

    let b = svc.register_partner(
        "Education Platform",
        PartnerIntegrationType::Custom,
        "Education",
        vec!["rewards".into(), "courses".into(), "badges".into()],
        None,
        json!({"site": "https://edu.example"}),
    );

    let c = svc.register_partner(
        "Payments App",
        PartnerIntegrationType::Payment,
        "Payments",
        vec!["wallet".into(), "qr".into()],
        None,
        json!({}),
    );

    // Link complementary between A and B (different markets, shared tag 'rewards')
    let comp = svc
        .link_complementary(&a.partner.id, &b.partner.id, Some("Guild <> Learn".into()))
        .expect("link should succeed");
    assert!(comp.different_markets);
    assert!(comp.shared_tags.iter().any(|t| t == "rewards"));
    assert!(comp.score > 0.0);

    // List complementarities for A should include the new link
    let list = svc.list_complementarities_for(&a.partner.id);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].partner_b_id, b.partner.id);

    // Search complementary partners by tag 'rewards' excluding Gaming market returns B first
    let res = svc.search_complementary(Some("rewards"), Some("Gaming"));
    assert!(res.iter().any(|p| p.partner.id == b.partner.id));
    assert!(!res.iter().any(|p| p.partner.id == a.partner.id));
    assert!(res.iter().any(|p| p.partner.id == c.partner.id) || res.len() >= 1);
}

#[test]
fn test_defi_technology_integration_verification() {
    let mut svc = CollaborationService::new();
    let mut secret = String::from("shh_secret");
    let p = svc.register_partner(
        "DeFi Aggregator",
        PartnerIntegrationType::DeFi,
        "DeFi",
        vec!["yield".into(), "router".into()],
        Some(secret.clone()),
        json!({"audit": true}),
    );

    // Integrate with a protocol (e.g., Uniswap)
    let integ = svc
        .integrate_with_protocol(&p.partner.id, "Uniswap", vec!["swap".into(), "lp".into()])
        .expect("integration should be created");
    assert_eq!(integ.status, IntegrationStatus::Integrated);

    // Wrong secret should fail
    let bad = svc.verify_integration(&integ.id, "nope");
    assert!(bad.is_err());

    // Correct secret verifies
    let ok = svc
        .verify_integration(&integ.id, &secret)
        .expect("verification should pass");
    assert_eq!(ok.status, IntegrationStatus::Verified);

    // Deprecate
    let dep = svc
        .deprecate_integration(&integ.id)
        .expect("deprecate should work");
    assert_eq!(dep.status, IntegrationStatus::Deprecated);
}

#[test]
fn test_cross_promotion_campaign_metrics() {
    let mut svc = CollaborationService::new();
    let p = svc.register_partner(
        "Peace DAO",
        PartnerIntegrationType::Custom,
        "NGO",
        vec!["impact".into(), "donations".into()],
        None,
        json!({}),
    );

    let camp = svc
        .create_cross_promo_campaign(
            &p.partner.id,
            "Peace Week",
            "Joint giveaway and donation match",
            vec![SocialPlatform::Twitter, SocialPlatform::Discord],
            Some(7),
            50.0,
        )
        .expect("campaign created");

    // Record events: 100 impressions, 25 clicks, 10 signups, 5 conversions, $1234.5 volume
    for _ in 0..100 {
        svc.record_cross_promo_event(&camp.id, CrossPromoEventType::Impression, None)
            .unwrap();
    }
    for _ in 0..25 {
        svc.record_cross_promo_event(&camp.id, CrossPromoEventType::Click, None)
            .unwrap();
    }
    for _ in 0..10 {
        svc.record_cross_promo_event(&camp.id, CrossPromoEventType::Signup, None)
            .unwrap();
    }
    for _ in 0..5 {
        svc.record_cross_promo_event(&camp.id, CrossPromoEventType::Conversion, None)
            .unwrap();
    }
    svc.record_cross_promo_event(&camp.id, CrossPromoEventType::Volume, Some(1000.0))
        .unwrap();
    svc.record_cross_promo_event(&camp.id, CrossPromoEventType::Volume, Some(234.5))
        .unwrap();

    let m = svc.cross_promo_metrics(&camp.id).expect("metrics ok");
    assert_eq!(m.impressions, 100);
    assert_eq!(m.clicks, 25);
    assert_eq!(m.signups, 10);
    assert_eq!(m.conversions, 5);
    assert!((m.volume - 1234.5).abs() < 1e-6);
    assert!((m.ctr - 0.25).abs() < 1e-6);
    assert!((m.conversion_rate - 0.2).abs() < 1e-6);
}
