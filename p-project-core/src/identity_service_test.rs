use super::identity_service::*;

#[test]
fn mint_passport_and_verify() {
    let mut svc = IdentityService::new(IdentityServiceConfig::default());
    let profile_id = svc
        .register_profile(
            "Amal".into(),
            "0xamal".into(),
            Some("Peace Alliance".into()),
            Some("Diplomatic youth".into()),
        )
        .unwrap();

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("rank".into(), "peacemaker".into());
    let passport_id = svc
        .mint_peace_passport(profile_id.clone(), "ipfs://token".into(), metadata.clone())
        .unwrap();
    let passport = svc.get_passport(&passport_id).unwrap();
    assert_eq!(passport.wallet_address, "0xamal");
    assert!(!passport.verified_on_chain);

    svc.verify_passport(&passport_id, "0xproof".into()).unwrap();
    let verified = svc.get_passport(&passport_id).unwrap();
    assert!(verified.verified_on_chain);
    assert_eq!(verified.verification_tx.as_deref(), Some("0xproof"));
}

#[test]
fn record_contributions_and_scores() {
    let mut svc = IdentityService::new(IdentityServiceConfig::default());
    let profile_id = svc
        .register_profile(
            "Nadia".into(),
            "0xnadia".into(),
            None,
            Some("On-the-ground organizer".into()),
        )
        .unwrap();

    let contribution_id = svc
        .record_contribution(
            profile_id.clone(),
            250.0,
            "P".into(),
            "Clean water kit".into(),
            ContributionKind::Material,
        )
        .unwrap();
    let contribution = svc.get_contribution(&contribution_id).unwrap();
    assert_eq!(contribution.amount, 250.0);
    let profile = svc.get_profile(&profile_id).unwrap();
    assert_eq!(profile.contributions_total, 250.0);
    assert!(profile.humanitarian_score > 0.0);
}

#[test]
fn volunteer_hours_verified_on_chain() {
    let mut svc = IdentityService::new(IdentityServiceConfig::default());
    let profile_id = svc
        .register_profile(
            "Lina".into(),
            "0xlina".into(),
            Some("Youth Corps".into()),
            None,
        )
        .unwrap();

    let entry_id = svc
        .log_volunteer_hours(
            profile_id.clone(),
            "Patrol".into(),
            6.0,
            Some("Border".into()),
        )
        .unwrap();
    let entry = svc.get_volunteer_entry(&entry_id).unwrap();
    assert_eq!(entry.status, VolunteerHourStatus::Logged);

    svc.verify_volunteer_hours(&entry_id, "0xtxn".into())
        .unwrap();
    let verified_entry = svc.get_volunteer_entry(&entry_id).unwrap();
    assert_eq!(verified_entry.status, VolunteerHourStatus::Verified);
    assert_eq!(verified_entry.tx_hash.as_deref(), Some("0xtxn"));
    let profile = svc.get_profile(&profile_id).unwrap();
    assert!(profile.volunteer_hours_total >= 6.0);
}
