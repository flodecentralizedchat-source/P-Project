#[cfg(test)]
mod tests {
    use super::super::dao_governance_service::*;
    use serde_json::json;

    fn build_service() -> DaoGovernanceService {
        let config = DaoGovernanceConfig {
            required_quorum: 3,
            vote_duration_days: 1,
            majority_percentage: 0.6,
        };
        DaoGovernanceService::new(config)
    }

    #[test]
    fn test_create_proposals_for_all_types() {
        let mut service = build_service();

        let proposal_types = vec![
            DaoProposalType::CharityAllocation,
            DaoProposalType::TreasuryDecision,
            DaoProposalType::PartnershipExpansion,
            DaoProposalType::NewNgo,
        ];

        for (idx, ptype) in proposal_types.into_iter().enumerate() {
            let details = json!({ "request_id": idx });
            let id = service
                .create_proposal(
                    format!("Title {}", idx),
                    "Description".to_string(),
                    ptype,
                    Some(details.clone()),
                    format!("creator_{}", idx),
                )
                .unwrap();
            let proposal = service.get_proposal(&id).unwrap();
            assert_eq!(proposal.status, DaoProposalStatus::Active);
            assert_eq!(proposal.target_details, details);
        }
    }

    #[test]
    fn test_vote_tally_and_execution() {
        let mut service = build_service();
        let id = service
            .create_proposal(
                "Charity allocation".to_string(),
                "Fund children".to_string(),
                DaoProposalType::CharityAllocation,
                None,
                "leader".to_string(),
            )
            .unwrap();

        service
            .cast_vote(&id, "voter1".to_string(), VoteChoice::For)
            .unwrap();
        service
            .cast_vote(&id, "voter2".to_string(), VoteChoice::For)
            .unwrap();
        service
            .cast_vote(&id, "voter3".to_string(), VoteChoice::Against)
            .unwrap();

        let status = service.tally_votes(&id).unwrap();
        assert_eq!(status, DaoProposalStatus::Passed);
        assert_eq!(
            service.get_proposal(&id).unwrap().status,
            DaoProposalStatus::Passed
        );

        service.execute_proposal(&id).unwrap();
        assert_eq!(
            service.get_proposal(&id).unwrap().status,
            DaoProposalStatus::Executed
        );
        assert!(service.get_proposal(&id).unwrap().executed_at.is_some());
    }

    #[test]
    fn test_quorum_and_majority_requirements() {
        let mut service = build_service();
        let id = service
            .create_proposal(
                "Treasury decision".to_string(),
                "Allocate reserves".to_string(),
                DaoProposalType::TreasuryDecision,
                None,
                "treasury".to_string(),
            )
            .unwrap();

        service
            .cast_vote(&id, "voter1".to_string(), VoteChoice::For)
            .unwrap();
        service
            .cast_vote(&id, "voter2".to_string(), VoteChoice::Abstain)
            .unwrap();

        assert!(service.tally_votes(&id).is_err());

        service
            .cast_vote(&id, "voter3".to_string(), VoteChoice::Against)
            .unwrap();

        let status = service.tally_votes(&id).unwrap();
        assert_eq!(status, DaoProposalStatus::Rejected);
    }

    #[test]
    fn test_vote_changes_are_allowed_before_tally() {
        let mut service = build_service();
        let id = service
            .create_proposal(
                "Partner with peace hub".to_string(),
                "Expand ecosystem".to_string(),
                DaoProposalType::PartnershipExpansion,
                None,
                "ecosystem".to_string(),
            )
            .unwrap();

        service
            .cast_vote(&id, "voter1".to_string(), VoteChoice::Against)
            .unwrap();
        service
            .cast_vote(&id, "voter1".to_string(), VoteChoice::For)
            .unwrap();

        service
            .cast_vote(&id, "voter2".to_string(), VoteChoice::For)
            .unwrap();
        service
            .cast_vote(&id, "voter3".to_string(), VoteChoice::For)
            .unwrap();

        let status = service.tally_votes(&id).unwrap();
        assert_eq!(status, DaoProposalStatus::Passed);
    }

    #[test]
    fn test_errors_for_missing_or_finalized_proposals() {
        let mut service = build_service();
        assert!(service.tally_votes("missing").is_err());
        assert!(service.execute_proposal("missing").is_err());

        let id = service
            .create_proposal(
                "New NGO".to_string(),
                "Support community allies".to_string(),
                DaoProposalType::NewNgo,
                None,
                "founder".to_string(),
            )
            .unwrap();

        service
            .cast_vote(&id, "voter1".to_string(), VoteChoice::For)
            .unwrap();
        service
            .cast_vote(&id, "voter2".to_string(), VoteChoice::For)
            .unwrap();
        service
            .cast_vote(&id, "voter3".to_string(), VoteChoice::For)
            .unwrap();

        service.tally_votes(&id).unwrap();
        assert!(service.execute_proposal(&id).is_ok());
        assert!(service.execute_proposal(&id).is_err());
    }
}
