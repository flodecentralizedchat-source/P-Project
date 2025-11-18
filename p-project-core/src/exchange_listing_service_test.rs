use crate::exchange_listing_service::*;

#[test]
fn default_features_cover_table_entries() {
    let svc = ExchangeListingService::new();
    let features = svc.list_features();
    assert!(features
        .iter()
        .any(|f| f.name == "DEX Listing" && f.feature_type == ListingFeatureType::Primary));
    assert!(features
        .iter()
        .any(|f| f.name == "CEX Listing" && f.feature_type == ListingFeatureType::Phase2));
    assert!(features.iter().any(|f| f.name == "DEX Launch Strategy"));
    assert!(features.iter().any(|f| f.name == "DEX Listing Guide"));
    assert!(features.iter().any(|f| f.name == "Listing Fees"));
}

#[test]
fn can_query_features_by_type() {
    let svc = ExchangeListingService::new();
    let primary = svc.features_by_type(ListingFeatureType::Primary);
    assert_eq!(primary.len(), 1);
    assert_eq!(primary[0].name, "DEX Listing");
}

#[test]
fn fee_plan_updates_respect_bounds() {
    let mut svc = ExchangeListingService::new();
    let result = svc.update_fee_plan(
        250.0,
        3500.0,
        FeeStatus::Reserved,
        Some("reserving launch fees".into()),
    );
    assert!(result.is_ok());
    let plan = result.unwrap();
    assert_eq!(plan.min_usd, 250.0);
    assert_eq!(plan.max_usd, 3500.0);
    assert_eq!(plan.status, FeeStatus::Reserved);
    assert_eq!(plan.notes, "reserving launch fees");

    let bad = svc.update_fee_plan(500.0, 250.0, FeeStatus::Planned, None);
    assert!(bad.is_err());
}

#[test]
fn add_custom_feature_then_find_by_name() {
    let mut svc = ExchangeListingService::new();
    let custom = ExchangeListingFeature {
        id: "custom-dex".into(),
        name: "Alameda DEX".into(),
        feature_type: ListingFeatureType::Efficiency,
        details: "Test new L2".into(),
        purpose: "Cost test".into(),
        platform: Some("Monkol-L2".into()),
        created_at: chrono::Utc::now().naive_utc(),
    };
    svc.add_feature(custom.clone());
    let found = svc
        .get_feature_by_name("Alameda DEX")
        .expect("should find custom feature");
    assert_eq!(found.platform.unwrap(), "Monkol-L2");
    assert_eq!(found.details, "Test new L2");
}
