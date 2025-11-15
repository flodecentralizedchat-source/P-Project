#[cfg(test)]
mod tests {
    use super::super::digital_governance_service::*;
    use chrono::Duration;
    use serde_json::json;

    fn build_service() -> DigitalGovernanceService {
        let config = DigitalGovernanceConfig {
            required_quorum: 2,
            majority_threshold: 0.5,
            vote_duration_days: 1,
            max_district_funding: 1000.0,
        };
        DigitalGovernanceService::new(config)
    }

    #[test]
    fn schedule_and_conclude_town_hall() {
        let mut service = build_service();
        let when = DigitalGovernanceService::now() + Duration::days(1);
        let id = service
            .schedule_town_hall(
                "Virtual Town Hall".to_string(),
                "District 9".to_string(),
                "Mayor".to_string(),
                vec!["Security".to_string(), "Education".to_string()],
                when,
                45,
            )
            .unwrap();

        service.record_town_hall_attendance(&id, 120).unwrap();
        service
            .conclude_town_hall(
                &id,
                Some("Great turnout".to_string()),
                Some("https://recording".to_string()),
            )
            .unwrap();

        let hall = service.get_town_hall(&id).unwrap();
        assert_eq!(hall.attendee_count, 120);
        assert!(!hall.is_active);
        assert_eq!(hall.summary.as_deref(), Some("Great turnout"));
    }

    #[test]
    fn cast_and_tally_onchain_votes() {
        let mut service = build_service();
        let vote_id = service
            .create_onchain_vote(
                "Budget".to_string(),
                "Approve local budget".to_string(),
                "District 2".to_string(),
                Some(json!({"goal": "parks"})),
            )
            .unwrap();

        service
            .cast_vote(&vote_id, "alice".to_string(), VoteChoice::For)
            .unwrap();
        service
            .cast_vote(&vote_id, "bob".to_string(), VoteChoice::Against)
            .unwrap();
        service
            .cast_vote(&vote_id, "carol".to_string(), VoteChoice::For)
            .unwrap();

        let status = service.tally_votes(&vote_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);
        assert_eq!(
            service.get_vote(&vote_id).unwrap().status,
            ProposalStatus::Passed
        );
    }

    #[test]
    fn quorum_enforced_on_votes() {
        let mut service = build_service();
        let vote_id = service
            .create_onchain_vote(
                "Transport".to_string(),
                "Approve tram line".to_string(),
                "District 6".to_string(),
                None,
            )
            .unwrap();

        service
            .cast_vote(&vote_id, "alice".to_string(), VoteChoice::For)
            .unwrap();

        assert!(service.tally_votes(&vote_id).is_err());
    }

    #[test]
    fn create_and_allocate_district_funding() {
        let mut service = build_service();
        let funding_id = service
            .create_funding_proposal(
                "District 3".to_string(),
                "Storm Relief".to_string(),
                "Rebuild shelters".to_string(),
                900.0,
            )
            .unwrap();

        service
            .allocate_district_funding(&funding_id, 400.0)
            .unwrap();
        service
            .allocate_district_funding(&funding_id, 500.0)
            .unwrap();

        let proposal = service.get_funding_proposal(&funding_id).unwrap();
        assert_eq!(proposal.amount_allocated, 900.0);
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }

    #[test]
    fn district_funding_errors() {
        let mut service = build_service();
        let funding_id = service
            .create_funding_proposal(
                "District 5".to_string(),
                "Bike lanes".to_string(),
                "Mobility".to_string(),
                800.0,
            )
            .unwrap();

        assert!(service
            .allocate_district_funding(&funding_id, 900.0)
            .is_err());
        service.decline_funding(&funding_id).unwrap();
        assert_eq!(
            service.get_funding_proposal(&funding_id).unwrap().status,
            ProposalStatus::Rejected
        );
    }
}
