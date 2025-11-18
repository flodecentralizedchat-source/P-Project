use super::{Treasury, TreasuryError};

#[test]
fn test_presale_whitelist_and_public_contribution() {
    let mut treasury = Treasury::new();
    treasury.configure_presale(350_000_000.0, 200_000.0);

    let presale = treasury.get_presale_program();
    let price = presale.price_per_token;
    assert!(price > 0.0);

    treasury.add_presale_whitelist("early");
    let tokens = treasury
        .contribute_to_presale("early", 1000.0)
        .expect("Should let whitelisted user contribute");
    assert!((tokens - (1000.0 / price)).abs() < 1e-6);
    assert_eq!(
        treasury.get_presale_program().contributions["early"],
        1000.0
    );

    let err = treasury.contribute_to_presale("late", 500.0);
    assert_eq!(err.unwrap_err(), TreasuryError::PresaleWhitelistRequired);

    treasury.open_presale_public_phase();
    let public_tokens = treasury.contribute_to_presale("late", 500.0).unwrap();
    assert!(public_tokens > 0.0);
    assert_eq!(treasury.get_reserve_balance("USD"), 1500.0);
}

#[test]
fn test_presale_target_limits_contribution() {
    let mut treasury = Treasury::new();
    treasury.configure_presale(350_000_000.0, 200_000.0);
    treasury.open_presale_public_phase();

    let tokens = treasury.contribute_to_presale("whale", 200_000.0).unwrap();
    assert!(tokens > 0.0);
    let err = treasury.contribute_to_presale("whale2", 1.0);
    assert_eq!(err.unwrap_err(), TreasuryError::PresaleTargetReached);
}

#[test]
fn test_reserve_lock_and_release_flow() {
    let mut treasury = Treasury::new();
    treasury.lock_reserve(350_000_000.0, "Crash buffer".to_string());
    assert!(treasury.is_reserve_locked());

    let released = treasury.release_reserve().unwrap();
    assert_eq!(released, 350_000_000.0 * 0.15);
    assert!(!treasury.is_reserve_locked());
    assert_eq!(
        treasury.release_reserve().unwrap_err(),
        TreasuryError::ReserveNotLocked
    );
}

#[test]
fn test_development_milestone_release_tracks_release_amount() {
    let mut treasury = Treasury::new();
    treasury.configure_development_fund(350_000_000.0);
    treasury.schedule_development_milestone(
        "Milestone Alpha".to_string(),
        "Launch automation".to_string(),
        1_000_000.0,
    );

    let remaining_before = treasury.get_development_fund().remaining();
    let released = treasury
        .release_development_milestone("Milestone Alpha")
        .unwrap();
    assert_eq!(released, 1_000_000.0);
    assert!(
        treasury.get_development_fund().released_amount >= released
            && treasury.get_development_fund().remaining() < remaining_before
    );

    assert_eq!(
        treasury
            .release_development_milestone("Milestone Alpha")
            .unwrap_err(),
        TreasuryError::DevelopmentMilestoneAlreadyReleased
    );

    assert_eq!(
        treasury
            .release_development_milestone("Missing")
            .unwrap_err(),
        TreasuryError::DevelopmentMilestoneNotFound
    );
}
