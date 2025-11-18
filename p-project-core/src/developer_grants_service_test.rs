#[cfg(test)]
mod tests {
    use super::super::developer_grants_service::*;
    use chrono::Utc;

    #[test]
    fn test_create_and_open_program() {
        let mut svc = DeveloperGrantsService::new();
        let start = Utc::now().naive_utc();
        let prog_id = svc
            .create_program(
                "Dev Grants Q1".to_string(),
                "Fund core builders".to_string(),
                100_000.0,
                Some(start),
                None,
                Some("OSS, core infra".to_string()),
            )
            .unwrap();

        assert!(prog_id.starts_with("grantprog_"));
        let prog = svc.programs.get(&prog_id).unwrap();
        assert_eq!(prog.total_budget, 100_000.0);
        assert_eq!(prog.allocated_budget, 0.0);
        assert_eq!(prog.status, ProgramStatus::Draft);

        svc.set_program_status(&prog_id, ProgramStatus::Open)
            .unwrap();
        assert_eq!(
            svc.programs.get(&prog_id).unwrap().status,
            ProgramStatus::Open
        );
    }

    #[test]
    fn test_application_review_and_approval_within_budget() {
        let mut svc = DeveloperGrantsService::new();
        let prog_id = svc
            .create_program(
                "Dev Grants".to_string(),
                "Builders".to_string(),
                50_000.0,
                None,
                None,
                None,
            )
            .unwrap();
        svc.set_program_status(&prog_id, ProgramStatus::Open)
            .unwrap();

        let app_id = svc
            .submit_application(
                &prog_id,
                "dev_1".to_string(),
                "Indexer".to_string(),
                "Fast indexer for events".to_string(),
                15_000.0,
                Some("https://example.com/proposal".to_string()),
            )
            .unwrap();

        svc.start_review(&app_id).unwrap();
        svc.add_review(&app_id, "r1".to_string(), 8, 9, 8, None)
            .unwrap();
        svc.add_review(&app_id, "r2".to_string(), 9, 8, 8, None)
            .unwrap();

        let app = svc.applications.get(&app_id).unwrap();
        assert!(app.average_score >= 8.0 && app.average_score <= 9.5);

        svc.decide_application(&app_id, true, None).unwrap();

        let prog = svc.programs.get(&prog_id).unwrap();
        assert_eq!(prog.allocated_budget, 15_000.0);
        let app = svc.applications.get(&app_id).unwrap();
        assert_eq!(app.status, ApplicationStatus::Approved);
        assert_eq!(app.awarded_amount, 15_000.0);
    }

    #[test]
    fn test_approval_exceeding_budget_fails() {
        let mut svc = DeveloperGrantsService::new();
        let prog_id = svc
            .create_program(
                "Small Budget".to_string(),
                "Builders".to_string(),
                10_000.0,
                None,
                None,
                None,
            )
            .unwrap();
        svc.set_program_status(&prog_id, ProgramStatus::Open)
            .unwrap();

        let app_id = svc
            .submit_application(
                &prog_id,
                "dev_2".to_string(),
                "Explorer".to_string(),
                "Block explorer".to_string(),
                12_000.0,
                None,
            )
            .unwrap();
        svc.start_review(&app_id).unwrap();

        let res = svc.decide_application(&app_id, true, None);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "insufficient program budget");
    }

    #[test]
    fn test_milestone_and_payout_flow() {
        let mut svc = DeveloperGrantsService::new();
        let prog_id = svc
            .create_program(
                "Dev Grants".to_string(),
                "Builders".to_string(),
                100_000.0,
                None,
                None,
                None,
            )
            .unwrap();
        svc.set_program_status(&prog_id, ProgramStatus::Open)
            .unwrap();

        let app_id = svc
            .submit_application(
                &prog_id,
                "dev_3".to_string(),
                "SDK".to_string(),
                "Developer SDK".to_string(),
                20_000.0,
                None,
            )
            .unwrap();
        svc.start_review(&app_id).unwrap();
        svc.decide_application(&app_id, true, Some(18_000.0))
            .unwrap();

        // Add milestones totaling <= awarded amount
        let m1 = svc
            .add_milestone(
                &app_id,
                "Alpha".to_string(),
                "Alpha release".to_string(),
                None,
                8_000.0,
            )
            .unwrap();
        let m2 = svc
            .add_milestone(
                &app_id,
                "Beta".to_string(),
                "Beta release".to_string(),
                None,
                10_000.0,
            )
            .unwrap();

        // Submit, approve, and pay first milestone
        svc.submit_milestone_proof(&app_id, &m1, "https://proof/alpha".to_string())
            .unwrap();
        svc.approve_milestone(&app_id, &m1).unwrap();
        let payout = svc.pay_milestone(&app_id, &m1).unwrap();
        assert_eq!(payout.amount, 8_000.0);
        assert!(payout.tx_hash.is_some());

        // Second milestone must be submitted before paying
        let pay_res = svc.pay_milestone(&app_id, &m2);
        assert!(pay_res.is_err());
        assert_eq!(
            pay_res.unwrap_err().to_string(),
            "milestone must be approved to pay"
        );
    }

    #[test]
    fn test_milestones_cannot_exceed_award() {
        let mut svc = DeveloperGrantsService::new();
        let prog_id = svc
            .create_program(
                "Caps".to_string(),
                "Builders".to_string(),
                30_000.0,
                None,
                None,
                None,
            )
            .unwrap();
        svc.set_program_status(&prog_id, ProgramStatus::Open)
            .unwrap();
        let app_id = svc
            .submit_application(
                &prog_id,
                "dev_4".to_string(),
                "Tooling".to_string(),
                "CLI tooling".to_string(),
                25_000.0,
                None,
            )
            .unwrap();
        svc.start_review(&app_id).unwrap();
        svc.decide_application(&app_id, true, Some(20_000.0))
            .unwrap();

        svc.add_milestone(&app_id, "M1".to_string(), "M1".to_string(), None, 10_000.0)
            .unwrap();
        let res = svc.add_milestone(&app_id, "M2".to_string(), "M2".to_string(), None, 12_000.0);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "milestones exceed awarded amount"
        );
    }
}
