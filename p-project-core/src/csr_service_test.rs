use super::csr_service::*;

#[test]
fn contribution_tracking_stores_records() {
    let mut svc = CsrService::new(CsrServiceConfig::default());
    let contribution_id = svc
        .record_contribution(
            "Solar Clean Energy".into(),
            "emp_1".into(),
            "Dana".into(),
            "0xdana".into(),
            700.0,
            "P".into(),
            Some("Community kit".into()),
        )
        .unwrap();
    let contribution = svc.get_contribution(&contribution_id).unwrap();
    assert_eq!(contribution.project, "Solar Clean Energy");
    assert_eq!(contribution.status, ContributionStatus::Recorded);
    assert_eq!(contribution.amount, 700.0);
}

#[test]
fn employee_good_deed_wallet_updates_points_and_balance() {
    let mut svc = CsrService::new(CsrServiceConfig {
        good_deed_point_value: 0.01,
        ..CsrServiceConfig::default()
    });
    let employee_id = svc
        .register_employee_wallet("emp_2".into(), "Zara".into(), "0xzara".into())
        .unwrap();
    svc.award_good_deed(&employee_id, "Mentored youth".into(), 120.0)
        .unwrap();
    svc.award_good_deed(&employee_id, "Food drive".into(), 30.0)
        .unwrap();
    let minted = svc.redeem_good_deed_points(&employee_id, 50.0).unwrap();
    assert_eq!(minted, 0.5);
    let wallet = svc.get_wallet(&employee_id).unwrap();
    assert!(wallet.points < 150.0);
    assert_eq!(wallet.coin_balance, 0.5);
    assert!(wallet.deeds.iter().any(|d| d.points == -50.0));
}

#[test]
fn donation_matching_campaign_restricts_budget() {
    let mut svc = CsrService::new(CsrServiceConfig::default());
    let campaign_id = svc
        .register_match_campaign("CSR Match".into(), 600.0, 0.5)
        .unwrap();
    let c1 = svc
        .record_contribution(
            "School upgrade".into(),
            "philanthropy".into(),
            "Corp".into(),
            "0xcorp".into(),
            400.0,
            "P".into(),
            None,
        )
        .unwrap();
    let c2 = svc
        .record_contribution(
            "Health clinic".into(),
            "philanthropy".into(),
            "Corp".into(),
            "0xcorp".into(),
            800.0,
            "P".into(),
            None,
        )
        .unwrap();

    let matched1 = svc.match_contribution(&campaign_id, &c1).unwrap();
    assert_eq!(matched1, 200.0);
    let matched2 = svc.match_contribution(&campaign_id, &c2).unwrap();
    assert_eq!(matched2, 400.0);
    let campaign = svc.get_campaign(&campaign_id).unwrap();
    assert_eq!(campaign.status, CampaignStatus::Completed);
    let contribution2 = svc.get_contribution(&c2).unwrap();
    assert_eq!(contribution2.status, ContributionStatus::Matched);
    assert!(svc.match_records.values().any(|r| r.contribution_id == c2));
}
