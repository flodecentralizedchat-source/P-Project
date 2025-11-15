#[cfg(test)]
mod tests {
    use super::super::sponsorship_service::*;

    fn build_service() -> SponsorshipService {
        let config = SponsorshipServiceConfig {
            currency: "P".to_string(),
            max_single_transaction: 10_000.0,
        };
        SponsorshipService::new(config)
    }

    #[test]
    fn test_student_registration_and_sponsorship() {
        let mut service = build_service();

        let student_id = service
            .register_student(
                "Amina J.".to_string(),
                "South Sudan".to_string(),
                true,
                "International Relations".to_string(),
                "Undergraduate".to_string(),
                1_000.0,
                "P".to_string(),
                Some("Needs tuition for peace studies".to_string()),
            )
            .unwrap();

        let request = SponsorshipRequest {
            target_id: student_id.clone(),
            sponsor_id: "sponsor_alpha".to_string(),
            amount: 800.0,
            currency: "P".to_string(),
            message: Some("Investing in peace".to_string()),
        };

        let tx = service.sponsor_student(request).unwrap();

        let student = service.get_student(&student_id).unwrap();
        assert_eq!(student.funded_amount, 800.0);
        assert_eq!(student.sponsor_count, 1);
        assert!(student.is_active);
        assert_eq!(tx.amount, 800.0);
        assert_eq!(tx.target_type, SponsorshipTargetType::Student);
        assert!(tx.transaction_id.starts_with("sponsor_"));
        assert_eq!(
            service
                .get_transaction(&tx.transaction_id)
                .unwrap()
                .transaction_id,
            tx.transaction_id
        );
    }

    #[test]
    fn test_student_becomes_fully_funded() {
        let mut service = build_service();

        let student_id = service
            .register_student(
                "Rania S.".to_string(),
                "Syria".to_string(),
                true,
                "Peace & Conflict Resolution".to_string(),
                "Master's".to_string(),
                1_500.0,
                "P".to_string(),
                None,
            )
            .unwrap();

        let first_request = SponsorshipRequest {
            target_id: student_id.clone(),
            sponsor_id: "sponsor_beta".to_string(),
            amount: 1_200.0,
            currency: "P".to_string(),
            message: None,
        };
        service.sponsor_student(first_request).unwrap();

        let second_request = SponsorshipRequest {
            target_id: student_id.clone(),
            sponsor_id: "sponsor_gamma".to_string(),
            amount: 300.0,
            currency: "P".to_string(),
            message: Some("Final mile".to_string()),
        };
        let tx = service.sponsor_student(second_request).unwrap();

        let student = service.get_student(&student_id).unwrap();
        assert_eq!(student.funded_amount, 1_500.0);
        assert_eq!(student.sponsor_count, 2);
        assert!(!student.is_active);
        assert_eq!(tx.amount, 300.0);
    }

    #[test]
    fn test_peace_builder_and_program_funding() {
        let mut service = build_service();

        let builder_id = service
            .register_peace_builder(
                "Khadija B.".to_string(),
                "Kenya".to_string(),
                "Youth mediation".to_string(),
                8,
                2_000.0,
                "P".to_string(),
                Some("Empowering youth mediators".to_string()),
            )
            .unwrap();

        let builder_request = SponsorshipRequest {
            target_id: builder_id.clone(),
            sponsor_id: "sponsor_delta".to_string(),
            amount: 2_000.0,
            currency: "P".to_string(),
            message: None,
        };
        let builder_tx = service.sponsor_peace_builder(builder_request).unwrap();

        let builder = service.get_builder(&builder_id).unwrap();
        assert_eq!(builder.funded_amount, 2_000.0);
        assert_eq!(builder.sponsor_count, 1);
        assert!(!builder.is_active);
        assert_eq!(builder_tx.target_type, SponsorshipTargetType::PeaceBuilder);

        let program_id = service
            .register_program(
                "Borders Without Barrier".to_string(),
                "Global Peace Collective".to_string(),
                "West Africa".to_string(),
                "Curriculum on restorative justice".to_string(),
                5_000.0,
                "P".to_string(),
                120,
            )
            .unwrap();

        let program_request = SponsorshipRequest {
            target_id: program_id.clone(),
            sponsor_id: "sponsor_epsilon".to_string(),
            amount: 1_500.0,
            currency: "P".to_string(),
            message: Some("Seed funding".to_string()),
        };
        let program_tx = service.fund_program(program_request).unwrap();

        let program = service.get_program(&program_id).unwrap();
        assert_eq!(program.funds_received, 1_500.0);
        assert!(program.is_active);
        assert_eq!(
            program_tx.target_type,
            SponsorshipTargetType::EducationProgram
        );
    }

    #[test]
    fn test_error_conditions() {
        let mut service = build_service();

        // Unsupported currency
        let result = service.register_student(
            "Maya D.".to_string(),
            "Lebanon".to_string(),
            true,
            "Journalism".to_string(),
            "Bachelor".to_string(),
            800.0,
            "USD".to_string(),
            None,
        );
        assert!(result.is_err());

        let student_id = service
            .register_student(
                "Maya D.".to_string(),
                "Lebanon".to_string(),
                true,
                "Journalism".to_string(),
                "Bachelor".to_string(),
                800.0,
                "P".to_string(),
                None,
            )
            .unwrap();

        let over_amount_request = SponsorshipRequest {
            target_id: student_id.clone(),
            sponsor_id: "budgetary".to_string(),
            amount: 20_000.0,
            currency: "P".to_string(),
            message: None,
        };
        assert!(service.sponsor_student(over_amount_request).is_err());

        let invalid_currency_request = SponsorshipRequest {
            target_id: student_id.clone(),
            sponsor_id: "budgetary".to_string(),
            amount: 100.0,
            currency: "USD".to_string(),
            message: None,
        };
        assert!(service.sponsor_student(invalid_currency_request).is_err());

        let missing_request = SponsorshipRequest {
            target_id: "missing_student".to_string(),
            sponsor_id: "budgetary".to_string(),
            amount: 100.0,
            currency: "P".to_string(),
            message: None,
        };
        assert!(service.sponsor_student(missing_request).is_err());

        let builder_id = service
            .register_peace_builder(
                "Farid H.".to_string(),
                "Uganda".to_string(),
                "Community healing".to_string(),
                5,
                500.0,
                "P".to_string(),
                None,
            )
            .unwrap();

        let request_one = SponsorshipRequest {
            target_id: builder_id.clone(),
            sponsor_id: "shoule".to_string(),
            amount: 500.0,
            currency: "P".to_string(),
            message: None,
        };
        service.sponsor_peace_builder(request_one).unwrap();

        // Builder should no longer accept funds
        let request_two = SponsorshipRequest {
            target_id: builder_id,
            sponsor_id: "shoule".to_string(),
            amount: 5.0,
            currency: "P".to_string(),
            message: None,
        };
        assert!(service.sponsor_peace_builder(request_two).is_err());
    }
}
