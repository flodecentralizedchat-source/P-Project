#[cfg(test)]
mod tests {
    use super::super::nft_collectibles_service::*;
    use serde_json::json;

    fn build_service() -> NftCollectiblesService {
        let config = NftCollectiblesConfig {
            max_mints_per_owner: 3,
        };
        NftCollectiblesService::new(config)
    }

    #[test]
    fn mint_peace_hero_avatar_collects_metadata() {
        let mut service = build_service();
        let nft = service
            .mint_peace_hero_avatar(
                "Guardian of Water".to_string(),
                "owner1".to_string(),
                "Calm Storm".to_string(),
                "Shields waterways".to_string(),
                Some(json!({"artifact": "trident"})),
            )
            .unwrap();

        assert_eq!(nft.nft_type, PeaceNFTType::HeroAvatar);
        assert_eq!(nft.hero_power.as_deref(), Some("Calm Storm"));
        assert_eq!(service.list_nfts_by_owner("owner1").len(), 1);
    }

    #[test]
    fn mint_art_collection_tracks_ngo() {
        let mut service = build_service();
        let nft = service
            .mint_peace_art_collection(
                "Peaceful Horizons".to_string(),
                "owner2".to_string(),
                "Hope Now".to_string(),
                "Proceeds to NGO".to_string(),
                Some(json!({"pieces": 12})),
            )
            .unwrap();

        assert_eq!(nft.nft_type, PeaceNFTType::ArtCollection);
        assert_eq!(nft.supporting_ngo.as_deref(), Some("Hope Now"));
        assert_eq!(
            service
                .ngo_support_summary("Hope Now")
                .unwrap()
                .total_amount,
            0.0
        );
        service.donate_to_ngo_via_nft(&nft.id, 150.0).unwrap();
        let summary = service.ngo_support_summary("Hope Now").unwrap();
        assert_eq!(summary.total_amount, 150.0);
    }

    #[test]
    fn mint_medal_and_transfer() {
        let mut service = build_service();
        let nft = service
            .mint_medal_of_peace(
                "Amani".to_string(),
                "owner3".to_string(),
                "Gold".to_string(),
                "Medal for community peace building".to_string(),
                None,
            )
            .unwrap();

        assert_eq!(nft.nft_type, PeaceNFTType::MedalOfPeace);
        assert_eq!(nft.badge_level.as_deref(), Some("Gold"));

        service.transfer_nft(&nft.id, "owner4".to_string()).unwrap();
        let transferred = service.get_nft(&nft.id).unwrap();
        assert_eq!(transferred.owner_id, "owner4");
    }

    #[test]
    fn enforce_mint_limit() {
        let mut service = build_service();
        for i in 0..3 {
            let title = format!("Hero {}", i);
            service
                .mint_peace_hero_avatar(
                    title,
                    "owner_limit".to_string(),
                    "Power".to_string(),
                    "desc".to_string(),
                    None,
                )
                .unwrap();
        }
        assert!(service
            .mint_peace_hero_avatar(
                "One Too Many".to_string(),
                "owner_limit".to_string(),
                "Power".to_string(),
                "desc".to_string(),
                None,
            )
            .is_err());
    }

    #[test]
    fn invalid_donations_fail() {
        let mut service = build_service();
        let nft = service
            .mint_peace_hero_avatar(
                "Guardian".to_string(),
                "owner5".to_string(),
                "Light".to_string(),
                "desc".to_string(),
                None,
            )
            .unwrap();
        assert!(service.donate_to_ngo_via_nft(&nft.id, 10.0).is_err());
    }
}
