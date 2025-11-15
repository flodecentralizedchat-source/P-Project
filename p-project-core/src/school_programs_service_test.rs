use super::school_programs_service::*;

#[test]
fn peace_club_grant_flow() {
    let mut svc = SchoolProgramsService::new(SchoolProgramsConfig {
        currency: "P".into(),
        max_grant_amount: 5_000.0,
        reward_points_per_course: 150.0,
    });
    let school_id = svc
        .register_school(
            "Global Peace Academy".into(),
            "0xschoolpay".into(),
            Some("City".into()),
        )
        .unwrap();
    svc.verify_school(&school_id).unwrap();

    let club_id = svc
        .register_peace_club(
            school_id.clone(),
            "Campus Peace Club".into(),
            Some("Advocates for civic harmony".into()),
        )
        .unwrap();
    svc.verify_peace_club(&club_id).unwrap();

    let grant_id = svc
        .request_treasury_grant(club_id.clone(), 4_200.0, "Community water well".into())
        .unwrap();
    svc.approve_grant(&grant_id, Some("High impact".into()))
        .unwrap();
    svc.disburse_grant(&grant_id).unwrap();

    let grant = svc.get_grant(&grant_id).expect("grant should exist");
    assert_eq!(grant.status, GrantStatus::Disbursed);
    assert!(grant.tx_hash.is_some());
    assert_eq!(grant.currency, "P");
    assert_eq!(grant.club_id, club_id);
}

#[test]
fn student_reward_account_cycle() {
    let mut svc = SchoolProgramsService::new(SchoolProgramsConfig::default());
    let school_id = svc
        .register_school("Peace High".into(), "0xschool".into(), None)
        .unwrap();
    svc.verify_school(&school_id).unwrap();

    let student_id = "student_ava";
    svc.register_student_reward_account(
        student_id.into(),
        "Ava".into(),
        school_id.clone(),
        "0xava".into(),
    )
    .unwrap();

    let total = svc
        .award_reward_points(student_id, 80.0, "Peace club cleanup".into())
        .unwrap();
    assert_eq!(total, 80.0);

    svc.award_badge(student_id, "Peace Ambassador".into())
        .unwrap();
    svc.redeem_reward_points(student_id, 30.0, "Merit store".into())
        .unwrap();

    let account = svc.get_reward_account(student_id).unwrap();
    assert_eq!(account.points, 50.0);
    assert!(account.badges.contains(&"Peace Ambassador".to_string()));
    assert!(account
        .history
        .iter()
        .any(|tx| tx.kind == RewardKind::Earned));
}

#[test]
fn blockchain_course_integration_rewards() {
    let mut svc = SchoolProgramsService::new(SchoolProgramsConfig {
        currency: "P".into(),
        max_grant_amount: 10_000.0,
        reward_points_per_course: 200.0,
    });
    let school_id = svc
        .register_school("Blockchain University".into(), "0xschooluni".into(), None)
        .unwrap();
    svc.verify_school(&school_id).unwrap();

    let student_id = "student_lee";
    svc.register_student_reward_account(
        student_id.into(),
        "Lee".into(),
        school_id.clone(),
        "0xlee".into(),
    )
    .unwrap();

    let course_id = svc
        .create_blockchain_course(
            school_id.clone(),
            "Blockchain Foundations".into(),
            "Introductory track".into(),
            vec!["Consensus".into(), "Smart Contracts".into()],
            true,
        )
        .unwrap();

    let enrollment_id = svc
        .enroll_student_in_course(student_id.into(), course_id.clone())
        .unwrap();

    svc.complete_course_module(&enrollment_id, "Consensus")
        .unwrap();
    svc.complete_course_module(&enrollment_id, "Smart Contracts")
        .unwrap();

    let enrollment = svc
        .get_enrollment(&enrollment_id)
        .expect("enrollment must exist");
    assert!(enrollment.completed);
    assert!(enrollment.certificate.is_some());

    let account = svc.get_reward_account("student_lee").unwrap();
    assert_eq!(account.points, svc.config.reward_points_per_course);
}
