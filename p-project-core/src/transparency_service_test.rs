use super::transparency_service::{TransparencyService, UpdateCategory};

#[test]
fn record_and_list_updates_by_category_and_limit() {
    let mut svc = TransparencyService::new();

    let id1 = svc.record_update(
        "Weekly Dev Update".into(),
        "Implemented staking yield calculators".into(),
        "alice".into(),
        UpdateCategory::Development,
        vec!["dev".into(), "staking".into()],
        vec![],
    );
    let id2 = svc.record_update(
        "Treasury Q1 Report".into(),
        "Published allocation details".into(),
        "bob".into(),
        UpdateCategory::Treasury,
        vec!["finance".into()],
        vec!["https://example.com/report".into()],
    );
    assert!(id1.starts_with("upd_"));
    assert!(id2.starts_with("upd_"));

    let all = svc.list_updates(None, None);
    assert_eq!(all.len(), 2);

    let treasury_only = svc.list_updates(None, Some(UpdateCategory::Treasury));
    assert_eq!(treasury_only.len(), 1);
    assert_eq!(treasury_only[0].title, "Treasury Q1 Report");

    let limited = svc.list_updates(Some(1), None);
    assert_eq!(limited.len(), 1);
}
