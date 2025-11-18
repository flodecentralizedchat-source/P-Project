use chrono::{Duration, Utc};
use p_project_contracts::PProjectToken;

fn setup_token() -> PProjectToken {
    let mut token = PProjectToken::new(350_000_000.0, 0.01, 0.005);
    // Relax protections to keep tests deterministic
    token.set_bot_protection(false);
    token.set_max_transfer_limit(350_000_000.0); // effectively disable anti-whale for tests
    token.set_max_daily_transfer_percent(1.0); // 100% of supply per day for tests
    token
}

#[test]
fn scheduled_burns_execute_only_when_due() {
    let mut token = setup_token();

    let now = Utc::now().naive_utc();
    let past = now - Duration::hours(1);
    let future = now + Duration::hours(1);

    token.add_scheduled_burn(past, 12_345.0);
    token.add_scheduled_burn(future, 99_999.0);
    token.set_burn_schedule_enabled(true);

    let start_supply = token.get_total_supply();
    let burned = token.execute_scheduled_burns().expect("scheduled burns");

    assert_eq!(burned, 12_345.0);
    assert_eq!(token.get_total_supply(), start_supply - 12_345.0);

    // Validate event log shows execution
    let executed = token.get_event_log().iter().any(|e| {
        e.event_type == "SCHEDULED_BURN_EXECUTED" && (e.amount - 12_345.0).abs() < f64::EPSILON
    });
    assert!(executed, "missing SCHEDULED_BURN_EXECUTED event");
}

#[test]
fn scheduled_burns_respect_toggle() {
    let mut token = setup_token();
    let past = Utc::now().naive_utc() - Duration::hours(1);

    token.add_scheduled_burn(past, 7_000.0);
    token.set_burn_schedule_enabled(false); // explicitly disable

    let burned = token.execute_scheduled_burns().expect("scheduled burns");
    assert_eq!(burned, 0.0, "burns should not execute when disabled");
}

#[test]
fn milestone_burns_execute_on_threshold() {
    let mut token = setup_token();
    token.initialize_distribution(vec![
        ("user1".to_string(), 1_000_000.0),
        ("user2".to_string(), 1_000_000.0),
    ]);

    token.add_milestone_burn(
        "tx_two".to_string(),
        "transactions_count".to_string(),
        2.0,
        500.0,
    );

    // Two transfers to reach the milestone
    token
        .transfer("user1", "user2", 1_000.0)
        .expect("transfer 1");
    token.transfer("user2", "user1", 500.0).expect("transfer 2");

    let burned = token.check_milestone_burns().expect("milestone burns");
    assert_eq!(burned, 500.0);

    let executed = token.get_event_log().iter().any(|e| {
        e.event_type == "MILESTONE_BURN_EXECUTED" && (e.amount - 500.0).abs() < f64::EPSILON
    });
    assert!(executed, "missing MILESTONE_BURN_EXECUTED event");
}

#[test]
fn revenue_linked_burns_apply_percentage() {
    let mut token = setup_token();

    token.add_revenue_linked_burn("fees".to_string(), 10_000.0, 10.0); // burns 1,000
    token.add_revenue_linked_burn("staking".to_string(), 50_000.0, 15.0); // burns 7,500

    let start = token.get_total_supply();
    let burned = token
        .execute_revenue_linked_burns()
        .expect("revenue-linked burns");

    assert_eq!(burned, 1_000.0 + 7_500.0);
    assert_eq!(token.get_total_supply(), start - burned);

    let executed_count = token
        .get_event_log()
        .iter()
        .filter(|e| e.event_type == "REVENUE_BURN_EXECUTED")
        .count();
    assert_eq!(executed_count, 2);
}

#[test]
fn revenue_linked_burns_only_execute_once() {
    let mut token = setup_token();

    token.add_revenue_linked_burn("fees".to_string(), 20_000.0, 10.0);
    token.add_revenue_linked_burn("staking".to_string(), 40_000.0, 5.0);

    let first_burn = token.execute_revenue_linked_burns().unwrap();
    let expected = 20_000.0 * 0.10 + 40_000.0 * 0.05;
    assert_eq!(first_burn, expected);
    assert!(token
        .get_revenue_linked_burns()
        .iter()
        .all(|burn| burn.executed));

    // Running again should not burn additional tokens
    let second_burn = token.execute_revenue_linked_burns().unwrap();
    assert_eq!(second_burn, 0.0);
}

#[test]
fn scheduled_buybacks_execute_only_when_due_and_mark_executed() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(350_000_000.0, 0.01, 0.005);

    treasury.add_funds("USD".to_string(), 200_000.0).unwrap();
    let now = Utc::now().naive_utc();
    treasury
        .add_scheduled_buyback(now - Duration::hours(2), 100_000.0, 0.01)
        .unwrap();
    treasury
        .add_scheduled_buyback(now - Duration::minutes(30), 50_000.0, 0.005)
        .unwrap();
    treasury
        .add_scheduled_buyback(now + Duration::hours(5), 25_000.0, 0.01)
        .unwrap();

    treasury.set_auto_buyback_enabled(true);

    let tokens_bought = treasury
        .execute_scheduled_buybacks(&mut token, 0.01)
        .unwrap();
    let expected_tokens = 100_000.0 / 0.01 + 50_000.0 / 0.005;
    assert_eq!(tokens_bought, expected_tokens);

    let schedules = treasury.get_scheduled_buybacks();
    assert!(schedules[0].executed);
    assert!(schedules[1].executed);
    assert!(!schedules[2].executed);

    assert_eq!(treasury.get_balance("USD"), 200_000.0 - 150_000.0);
    assert_eq!(treasury.get_total_buybacks(), 150_000.0);
}

#[test]
fn scheduled_buybacks_skip_when_disabled() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(350_000_000.0, 0.01, 0.005);

    treasury.add_funds("USD".to_string(), 100_000.0).unwrap();
    treasury
        .add_scheduled_buyback(Utc::now().naive_utc() - Duration::hours(1), 50_000.0, 0.005)
        .unwrap();

    treasury.set_auto_buyback_enabled(false);

    let tokens_bought = treasury
        .execute_scheduled_buybacks(&mut token, 0.005)
        .unwrap();
    assert_eq!(tokens_bought, 0.0);
    assert_eq!(treasury.get_balance("USD"), 100_000.0);
    assert!(!treasury.get_scheduled_buybacks()[0].executed);
}
