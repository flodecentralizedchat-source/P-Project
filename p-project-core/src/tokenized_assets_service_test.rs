#[cfg(test)]
mod tests {
    use super::super::tokenized_assets_service::*;
    use serde_json::json;

    fn build_service() -> TokenizedImpactAssetService {
        let config = TokenizedImpactAssetConfig {
            supported_currency: "P".to_string(),
            max_carbon_batch_size: 500_000.0,
        };
        TokenizedImpactAssetService::new(config)
    }

    #[test]
    fn test_carbon_credit_batch_and_issuance() {
        let mut service = build_service();

        let batch_id = service
            .create_carbon_credit_batch(
                "Sahel Wind".to_string(),
                "Senegal".to_string(),
                "Verra".to_string(),
                2025,
                100_000.0,
                "P".to_string(),
                Some(365),
                Some(json!({"project": "Sahel Wind"})),
            )
            .unwrap();

        let issuance = service
            .issue_carbon_credit(&batch_id, "buyer_alpha".to_string(), 1000.0)
            .unwrap();

        assert_eq!(issuance.credits, 1000.0);
        assert_eq!(issuance.purchaser_id, "buyer_alpha");
        let batch = service.carbon_batches.get(&batch_id).unwrap();
        assert_eq!(batch.credits_allocated, 1000.0);
        assert!(batch.is_active);

        let tx = service
            .transactions
            .values()
            .find(|tx| tx.asset_type == ImpactAssetType::CarbonCredit)
            .unwrap();
        assert_eq!(tx.amount, 1000.0);
        assert_eq!(tx.target_id, batch_id);
    }

    #[test]
    fn test_charity_receipt_generation_and_redemption() {
        let mut service = build_service();

        let receipt_id = service
            .create_charity_receipt(
                "donor_peace".to_string(),
                "Libraries for All".to_string(),
                250.0,
                "P".to_string(),
                Some(json!({"impact": "education"})),
            )
            .unwrap();

        let receipt = service.charity_receipts.get(&receipt_id).unwrap();
        assert_eq!(receipt.donor_id, "donor_peace");
        assert_eq!(receipt.status, CharityReceiptStatus::Active);

        service.redeem_charity_receipt(&receipt_id).unwrap();
        let redeemed = service.charity_receipts.get(&receipt_id).unwrap();
        assert_eq!(redeemed.status, CharityReceiptStatus::Redeemed);

        let err = service.redeem_charity_receipt(&receipt_id).unwrap_err();
        assert!(err.to_string().contains("already redeemed"));
    }

    #[test]
    fn test_impact_bond_purchase_flow() {
        let mut service = build_service();

        let bond_id = service
            .create_impact_bond(
                "Peace Water Grid".to_string(),
                "Global Peace Bank".to_string(),
                500.0,
                0.08,
                "P".to_string(),
                720,
                5,
                Some(json!({"tier": "platinum"})),
            )
            .unwrap();

        let minted = service
            .purchase_impact_bond(&bond_id, "investor_one".to_string(), 2)
            .unwrap();

        assert_eq!(minted.len(), 2);
        assert_eq!(minted[0].owner_id, "investor_one");
        let bond = service.impact_bonds.get(&bond_id).unwrap();
        assert_eq!(bond.minted_supply, 2);
        assert!(bond.is_active);

        let owner_nfts = service.get_bond_nfts_for_owner("investor_one");
        assert_eq!(owner_nfts.len(), 2);

        let tx = service
            .transactions
            .values()
            .find(|tx| tx.asset_type == ImpactAssetType::ImpactBondNFT)
            .unwrap();
        assert_eq!(tx.amount, 500.0 * 2.0);
    }

    #[test]
    fn test_invalid_conditions() {
        let mut service = build_service();

        // unsupported currency for carbon
        let err = service
            .create_carbon_credit_batch(
                "Future Forest".to_string(),
                "Brazil".to_string(),
                "Gold Standard".to_string(),
                2026,
                50_000.0,
                "USD".to_string(),
                None,
                None,
            )
            .unwrap_err();
        assert!(err.to_string().contains("Only P is supported"));

        // issue more credits than available
        let batch_id = service
            .create_carbon_credit_batch(
                "Future Forest".to_string(),
                "Brazil".to_string(),
                "Gold Standard".to_string(),
                2026,
                2_000.0,
                "P".to_string(),
                None,
                None,
            )
            .unwrap();
        let err = service
            .issue_carbon_credit(&batch_id, "buyer_two".to_string(), 10_000.0)
            .unwrap_err();
        assert!(err.to_string().contains("Not enough credits"));

        // invalid bond parameters
        let err = service.create_impact_bond(
            "Zero Bond".to_string(),
            "Issuer".to_string(),
            0.0,
            0.05,
            "P".to_string(),
            365,
            3,
            None,
        );
        assert!(err.is_err());

        // oversubscribe bond
        let bond_id = service
            .create_impact_bond(
                "Solar Schools".to_string(),
                "Tenant".to_string(),
                100.0,
                0.05,
                "P".to_string(),
                365,
                1,
                None,
            )
            .unwrap();
        service
            .purchase_impact_bond(&bond_id, "investor_two".to_string(), 1)
            .unwrap();

        let err = service
            .purchase_impact_bond(&bond_id, "investor_three".to_string(), 1)
            .unwrap_err();
        assert!(
            err.to_string().contains("no longer active") || err.to_string().contains("Not enough")
        );
    }
}
