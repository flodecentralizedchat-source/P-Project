#[cfg(test)]
mod tests {
    use super::super::marketing_community_service::*;

    #[test]
    fn test_budget_allocation_and_spend_limits() {
        let mut svc = MarketingCommunityService::with_supply(1_000_000.0).unwrap();
        let marketing_allowed = svc.remaining_budget(MarketingCategory::MarketingBudget);
        assert_eq!(marketing_allowed, 50_000.0);

        svc.spend_budget(MarketingCategory::MarketingBudget, 10_000.0)
            .unwrap();
        assert_eq!(
            svc.remaining_budget(MarketingCategory::MarketingBudget),
            marketing_allowed - 10_000.0
        );

        let res = svc.spend_budget(MarketingCategory::MarketingBudget, 100_000.0);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "insufficient_budget".to_string());
    }

    #[test]
    fn test_community_rewards_awarded_and_capped() {
        let mut svc = MarketingCommunityService::with_supply(500_000.0).unwrap();
        let initial = svc.remaining_budget(MarketingCategory::CommunityRewards);
        svc.award_community_reward("user_a", 2_000.0).unwrap();
        assert_eq!(svc.reward_balance("user_a"), 2_000.0);
        assert!(svc.remaining_budget(MarketingCategory::CommunityRewards) < initial);

        let res = svc.award_community_reward("user_a", initial + 1.0);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "insufficient_budget".to_string());
    }

    #[test]
    fn test_influencer_campaign_flow() {
        let mut svc = MarketingCommunityService::with_supply(2_000_000.0).unwrap();
        let engaged_budget = 5_000.0;
        let campaign = svc
            .create_influencer_campaign(
                "@tinyinfluencer".into(),
                "Telegram".into(),
                engaged_budget,
                vec!["1 post".into(), "1 AMA".into()],
            )
            .unwrap();
        assert_eq!(campaign.status, CampaignStatus::Proposed);

        svc.update_campaign_status(&campaign.id, CampaignStatus::Running, None)
            .unwrap();
        let completed = svc
            .update_campaign_status(
                &campaign.id,
                CampaignStatus::Completed,
                Some("Delivered beta coverage".into()),
            )
            .unwrap();
        assert_eq!(completed.status, CampaignStatus::Completed);
        assert_eq!(completed.report.as_deref(), Some("Delivered beta coverage"));
        assert!(completed.completed_at.is_some());

        // Influencer strategy budget decreased
        let fallen_budget = svc.remaining_budget(MarketingCategory::InfluencerStrategy);
        assert!(fallen_budget <= 20_000.0);
    }

    #[test]
    fn test_organic_activity_records_latest_first() {
        let mut svc = MarketingCommunityService::with_supply(1_000_000.0).unwrap();
        svc.log_organic_activity(
            "Community Building".into(),
            "Discord".into(),
            vec!["AMA".into()],
        );
        svc.log_organic_activity("Forums".into(), "Reddit".into(), vec!["Guide".into()]);

        let activities = svc.list_organic_activity(2);
        assert_eq!(activities.len(), 2);
        assert_eq!(activities[0].channel, "Reddit");
        assert_eq!(activities[1].channel, "Discord");
    }

    #[test]
    fn test_community_proposals_voting_and_finalize() {
        let mut svc = MarketingCommunityService::with_supply(1_000_000.0).unwrap();
        let request = 5_000.0;
        let proposal_id = svc
            .submit_community_proposal("Community Fund".into(), "Pay for grants".into(), request)
            .unwrap();

        svc.vote_proposal(&proposal_id, true).unwrap();
        svc.vote_proposal(&proposal_id, false).unwrap();
        svc.vote_proposal(&proposal_id, true).unwrap();

        let finalized = svc.finalize_proposal(&proposal_id).unwrap();
        assert_eq!(finalized.status, ProposalStatus::Passed);
        assert_eq!(finalized.votes_for, 2);
        assert_eq!(finalized.votes_against, 1);
        let config = svc.get_config();
        assert!(
            svc.remaining_budget(MarketingCategory::CommunityDao)
                < config.total_supply * config.dao_fund_pct
        );
    }
}
