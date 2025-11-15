#[cfg(test)]
mod tests {
    use super::super::credit_service::{
        CreditScoreProfile, CreditService, CreditServiceConfig, LoanStatus, MicroLoan,
        NGORegistration, SocialImpactEvent,
    };
    use chrono::Utc;

    fn credit_config() -> CreditServiceConfig {
        CreditServiceConfig {
            min_credit_score: 60.0,
            max_loan_amount: 1000.0,
            collateral_ratio: 0.5,
            default_interest_rate: 0.1,
            base_score: 50.0,
            max_duration_days: 30,
        }
    }

    fn impact_event(id: &str, score: f64) -> SocialImpactEvent {
        SocialImpactEvent {
            id: id.to_string(),
            description: "Verified impact".to_string(),
            impact_score: score,
            verified_by: Some("ngo".to_string()),
            timestamp: Utc::now().naive_utc(),
        }
    }

    #[test]
    fn test_credit_score_increases_with_events() {
        let mut service = CreditService::new(credit_config());
        let baseline = service.get_credit_score("user_1");
        assert_eq!(baseline, 50.0);

        let score = service
            .add_social_impact_event("user_1", impact_event("evt_1", 15.0))
            .unwrap();
        assert!(score > baseline);
        assert!(score <= 100.0);

        let profile = service.get_credit_score("user_1");
        assert_eq!(score, profile);
    }

    #[test]
    fn test_request_loan_requires_credit_and_collateral() {
        let mut service = CreditService::new(credit_config());
        let ngo = service.register_ngo(NGORegistration {
            name: "Peace Builders".to_string(),
            region: "Global".to_string(),
            max_loan_amount: 500.0,
            approved: true,
        });

        // Insufficient credit score should fail
        let err = service
            .request_micro_loan("user_low", 200.0, 200.0, &ngo.id)
            .expect_err("should require higher credit score");
        assert!(err.to_string().contains("Credit score"));

        // Boost score with a social impact event
        service
            .add_social_impact_event("user_low", impact_event("evt_boost", 20.0))
            .unwrap();

        // Collateral too low
        let err = service
            .request_micro_loan("user_low", 200.0, 50.0, &ngo.id)
            .expect_err("should require sufficient collateral");
        assert!(err.to_string().contains("Collateral"));
    }

    #[test]
    fn test_successful_loan_and_repayment() {
        let mut service = CreditService::new(credit_config());
        let ngo = service.register_ngo(NGORegistration {
            name: "NGO Lenders".to_string(),
            region: "Africa".to_string(),
            max_loan_amount: 1000.0,
            approved: true,
        });

        service
            .add_social_impact_event("hero", impact_event("evt_hero", 30.0))
            .unwrap();

        let loan = service
            .request_micro_loan("hero", 250.0, 200.0, &ngo.id)
            .expect("loan should be granted");
        assert_eq!(loan.status, LoanStatus::Active);
        assert_eq!(loan.borrower_id, "hero");
        assert_eq!(loan.ngo_id, ngo.id);
        assert!(loan.total_due() > loan.principal_amount);

        let repaid = service
            .repay_micro_loan(&loan.loan_id, loan.total_due())
            .expect("repayment should succeed");
        assert_eq!(repaid.status, LoanStatus::Repaid);
        assert!(repaid.repaid_amount >= repaid.total_due());
    }

    #[test]
    fn test_get_loans_by_borrower() {
        let mut service = CreditService::new(credit_config());
        let ngo = service.register_ngo(NGORegistration {
            name: "NGO Lenders".to_string(),
            region: "Asia".to_string(),
            max_loan_amount: 1000.0,
            approved: true,
        });

        service
            .add_social_impact_event("multi", impact_event("evt_multi", 15.0))
            .unwrap();

        let loan_a = service
            .request_micro_loan("multi", 100.0, 100.0, &ngo.id)
            .unwrap();
        let loan_b = service
            .request_micro_loan("multi", 150.0, 120.0, &ngo.id)
            .unwrap();

        let list = service.get_loans_by_borrower("multi");
        assert_eq!(list.len(), 2);
        assert!(list
            .iter()
            .map(|loan| &loan.loan_id)
            .collect::<Vec<_>>()
            .contains(&&loan_a.loan_id));
        assert!(list
            .iter()
            .map(|loan| &loan.loan_id)
            .collect::<Vec<_>>()
            .contains(&&loan_b.loan_id));
    }
}
