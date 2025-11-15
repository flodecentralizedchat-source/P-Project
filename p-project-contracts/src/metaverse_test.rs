#![cfg(test)]

use crate::metaverse::{BuildingType, LandParcel, MetaverseError, PeaceIsland};
use crate::token::PProjectToken;

#[test]
fn test_buy_land_and_build_home() {
    let mut island = PeaceIsland::new(
        "Peace Island".to_string(),
        "island_treasury".to_string(),
        20,
    );
    island
        .add_parcel(LandParcel {
            id: "parcel-1".to_string(),
            coordinates: (12.0, 34.0),
            size_sqm: 120.0,
            price: 500.0,
            owner: None,
            buildings: Vec::new(),
            locked: false,
        })
        .unwrap();

    let mut token = PProjectToken::new(10000.0, 0.01, 0.005);
    token.initialize_distribution(vec![("alice".to_string(), 1000.0)]);

    let paid = island
        .buy_land("parcel-1", "alice", &mut token)
        .expect("Should succeed");
    assert_eq!(paid, 500.0);
    assert_eq!(token.get_balance("alice"), 500.0);
    assert_eq!(token.get_balance("island_treasury"), 500.0);

    island
        .build_structure(
            "parcel-1",
            "alice",
            BuildingType::Home,
            "Peace Home".to_string(),
        )
        .unwrap();

    let parcel = island.get_parcel("parcel-1").unwrap();
    assert_eq!(parcel.owner.as_deref(), Some("alice"));
    assert_eq!(parcel.buildings.len(), 1);
    assert_eq!(parcel.buildings[0].building_type, BuildingType::Home);
}

#[test]
fn test_build_requires_owner_and_limit() {
    let mut island = PeaceIsland::new(
        "Peace Island".to_string(),
        "island_treasury".to_string(),
        10,
    );
    island
        .add_parcel(LandParcel {
            id: "parcel-2".to_string(),
            coordinates: (5.0, 6.0),
            size_sqm: 200.0,
            price: 750.0,
            owner: Some("bob".to_string()),
            buildings: Vec::new(),
            locked: false,
        })
        .unwrap();

    let err = island
        .build_structure(
            "parcel-2",
            "alice",
            BuildingType::School,
            "Peace School".to_string(),
        )
        .unwrap_err();
    assert_eq!(err, MetaverseError::UnauthorizedOwner);

    for _ in 0..4 {
        island
            .build_structure(
                "parcel-2",
                "bob",
                BuildingType::Garden,
                "Community Garden".to_string(),
            )
            .unwrap();
    }

    let err = island
        .build_structure(
            "parcel-2",
            "bob",
            BuildingType::Garden,
            "Extra Garden".to_string(),
        )
        .unwrap_err();
    assert_eq!(err, MetaverseError::BuildLimitReached);
}
