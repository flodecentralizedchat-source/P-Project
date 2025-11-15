#[cfg(test)]
mod tests {
    use super::super::government_service::*;
    use serde_json::json;

    fn build_service() -> GovernmentService {
        let config = GovernmentServiceConfig {
            supported_currency: "P".to_string(),
            max_disbursement: 500.0,
        };
        GovernmentService::new(config)
    }

    #[test]
    fn test_program_creation_and_incentive_award() {
        let mut service = build_service();

        let program_id = service
            .create_program(
                "Peace Labs".to_string(),
                "Reward peace inventions".to_string(),
                "Global".to_string(),
                GovernmentProgramType::PeaceIncentive,
                10_000.0,
                "P".to_string(),
                Some(json!({"theme": "innovation"})),
            )
            .unwrap();

        service.award_incentive(&program_id, 500.0).unwrap();
        let program = service.get_program(&program_id).unwrap();
        assert_eq!(program.distributed, 500.0);
        assert_eq!(program.program_type, GovernmentProgramType::PeaceIncentive);
    }

    #[test]
    fn test_youth_grant_submission_approval_rejection() {
        let mut service = build_service();

        let program_id = service
            .create_program(
                "Youth Futures".to_string(),
                "Grants for young changemakers".to_string(),
                "Region 5".to_string(),
                GovernmentProgramType::YouthGrant,
                20_000.0,
                "P".to_string(),
                None,
            )
            .unwrap();

        let application_id = service
            .submit_youth_grant_application(
                "applicant_1".to_string(),
                program_id.clone(),
                2_000.0,
                Some("Community radio pilot".to_string()),
            )
            .unwrap();

        service
            .approve_youth_grant(&application_id, 2_000.0)
            .unwrap();
        let application = service.get_application(&application_id).unwrap();
        assert_eq!(application.status, YouthGrantStatus::Approved);
        assert_eq!(application.amount_awarded, 2_000.0);

        // another application rejected
        let second_app = service
            .submit_youth_grant_application(
                "applicant_2".to_string(),
                program_id.clone(),
                500.0,
                None,
            )
            .unwrap();
        service
            .reject_youth_grant(&second_app, Some("Incomplete docs".to_string()))
            .unwrap();
        let rejected = service.get_application(&second_app).unwrap();
        assert_eq!(rejected.status, YouthGrantStatus::Rejected);
        assert_eq!(rejected.narrative.as_deref(), Some("Incomplete docs"));
    }

    #[test]
    fn test_welfare_disbursement_and_transparency_log() {
        let mut service = build_service();

        let program_id = service
            .create_program(
                "Refugee Welfare".to_string(),
                "Basic income support".to_string(),
                "Camp A".to_string(),
                GovernmentProgramType::WelfareDisbursement,
                5_000.0,
                "P".to_string(),
                None,
            )
            .unwrap();

        let disbursement_id = service
            .disburse_welfare_funds(
                &program_id,
                "beneficiary1".to_string(),
                250.0,
                "0xwallet123".to_string(),
                Some(json!({"purpose": "food"})),
            )
            .unwrap();

        let record = service.get_disbursement(&disbursement_id).unwrap();
        assert_eq!(record.amount, 250.0);
        assert_eq!(record.recipient_id, "beneficiary1");

        let log_id = service
            .log_transparency_event(
                &program_id,
                "Tracked disbursement".to_string(),
                "hash123".to_string(),
                "ok".to_string(),
            )
            .unwrap();

        let events = service.transparency_feed();
        assert!(events.iter().any(|evt| evt.event_id == log_id));
    }

    #[test]
    fn test_invalid_operations() {
        let mut service = build_service();

        // invalid currency
        let err = service.create_program(
            "Bad Program".to_string(),
            "Wrong money".to_string(),
            "Region".to_string(),
            GovernmentProgramType::PeaceIncentive,
            100.0,
            "USD".to_string(),
            None,
        );
        assert!(err.is_err());

        let program_id = service
            .create_program(
                "Youth".to_string(),
                "Kids".to_string(),
                "Region".to_string(),
                GovernmentProgramType::YouthGrant,
                1_000.0,
                "P".to_string(),
                None,
            )
            .unwrap();

        // awarding on wrong type
        assert!(service.award_incentive(&program_id, 100.0).is_err());

        let app_id = service
            .submit_youth_grant_application("app".to_string(), program_id.clone(), 200.0, None)
            .unwrap();

        // double approval fail - approve with too much
        assert!(service.approve_youth_grant(&app_id, 500.0).is_err());
        assert!(service.reject_youth_grant("missing", None).is_err());

        let welfare_id = service
            .create_program(
                "Welfare".to_string(),
                "fund".to_string(),
                "Region".to_string(),
                GovernmentProgramType::WelfareDisbursement,
                500.0,
                "P".to_string(),
                None,
            )
            .unwrap();

        // over disbursement
        assert!(service
            .disburse_welfare_funds(
                &welfare_id,
                "r".to_string(),
                1000.0,
                "wallet".to_string(),
                None,
            )
            .is_err());

        // transparency log on unknown program
        assert!(service
            .log_transparency_event(
                "missing",
                "desc".to_string(),
                "hash".to_string(),
                "ok".to_string()
            )
            .is_err());
    }
}
