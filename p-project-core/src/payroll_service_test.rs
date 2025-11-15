use super::payroll_service::*;

#[test]
fn peace_org_payroll_flow() {
    let mut svc = PayrollService::new(PayrollServiceConfig {
        currency: "P".into(),
        fee_percentage: 1.0,
        max_single_payout: 1_000_000.0,
    });
    let org_id = svc
        .register_organization(
            "PeaceOrg A".into(),
            OrganizationType::PeaceOrganization,
            "0xorg".into(),
        )
        .unwrap();
    svc.verify_organization(&org_id).unwrap();

    let e1 = svc
        .add_employee(
            org_id.clone(),
            "Alice".into(),
            "0xalice".into(),
            1000.0,
            "P".into(),
        )
        .unwrap();
    let _e2 = svc
        .add_employee(
            org_id.clone(),
            "Bob".into(),
            "0xbob".into(),
            2000.0,
            "P".into(),
        )
        .unwrap();
    assert!(svc.employees.get(&e1).is_some());

    let _sched = svc
        .create_pay_schedule(org_id.clone(), Frequency::Monthly)
        .unwrap();
    let run_id = svc.run_payroll(org_id.clone()).unwrap();
    let run = svc.get_pay_run(&run_id).unwrap();
    assert_eq!(run.status, PayRunStatus::Completed);
    assert_eq!(run.receipts.len(), 2);
    for r in &run.receipts {
        assert_eq!(r.currency, "P");
        assert_eq!(r.kind, ReceiptKind::Payroll);
        assert!(matches!(r.status, PaymentStatus::Completed));
        assert!(r.tx_hash.as_ref().unwrap().starts_with("0x"));
    }
}

#[test]
fn school_invoice_and_payment() {
    let mut svc = PayrollService::new(PayrollServiceConfig::default());
    let school_id = svc
        .register_organization(
            "Peace School".into(),
            OrganizationType::School,
            "0xschool".into(),
        )
        .unwrap();
    svc.verify_organization(&school_id).unwrap();
    let student_id = svc
        .add_student(school_id.clone(), "Charlie".into(), "0xcharlie".into())
        .unwrap();

    let items = vec![
        InvoiceItem {
            item_type: InvoiceItemType::Tuition,
            description: "Semester Tuition".into(),
            amount: 1500.0,
        },
        InvoiceItem {
            item_type: InvoiceItemType::Books,
            description: "Books".into(),
            amount: 200.0,
        },
    ];
    let inv_id = svc
        .create_invoice(school_id.clone(), student_id.clone(), items, "P".into())
        .unwrap();
    let inv_before = svc.get_invoice(&inv_id).unwrap();
    assert_eq!(inv_before.status, InvoiceStatus::Pending);
    assert_eq!(inv_before.total_amount, 1700.0);

    let receipt = svc.pay_invoice(inv_id.clone(), "0xpayer".into()).unwrap();
    assert_eq!(receipt.currency, "P");
    assert_eq!(receipt.kind, ReceiptKind::TuitionPayment);
    assert!(matches!(receipt.status, PaymentStatus::Completed));

    let inv_after = svc.get_invoice(&inv_id).unwrap();
    assert_eq!(inv_after.status, InvoiceStatus::Paid);
    assert!(inv_after.paid_at.is_some());
    assert!(inv_after.payment_receipt_id.is_some());
}

#[test]
fn volunteer_stipend_disbursement() {
    let mut svc = PayrollService::new(PayrollServiceConfig {
        currency: "P".into(),
        fee_percentage: 0.5,
        max_single_payout: 100_000.0,
    });
    let vol_org_id = svc
        .register_organization(
            "VolOrg".into(),
            OrganizationType::VolunteerOrganization,
            "0xvolorg".into(),
        )
        .unwrap();
    svc.verify_organization(&vol_org_id).unwrap();
    let _v1 = svc
        .add_volunteer(
            vol_org_id.clone(),
            "Dana".into(),
            "0xdana".into(),
            500.0,
            "P".into(),
            Frequency::Monthly,
        )
        .unwrap();
    let _v2 = svc
        .add_volunteer(
            vol_org_id.clone(),
            "Eli".into(),
            "0xeli".into(),
            300.0,
            "P".into(),
            Frequency::Weekly,
        )
        .unwrap();

    let receipts = svc.run_stipend_disbursement(vol_org_id.clone()).unwrap();
    assert_eq!(receipts.len(), 2);
    for r in receipts {
        assert_eq!(r.currency, "P");
        assert_eq!(r.kind, ReceiptKind::Stipend);
        assert!(matches!(r.status, PaymentStatus::Completed));
    }
}

#[test]
fn validations_and_errors() {
    let mut svc = PayrollService::new(PayrollServiceConfig::default());
    let org_id = svc
        .register_organization(
            "OrgX".into(),
            OrganizationType::PeaceOrganization,
            "0xorg".into(),
        )
        .unwrap();

    // Not verified yet
    assert!(svc
        .add_employee(
            org_id.clone(),
            "Emp".into(),
            "0xemp".into(),
            100.0,
            "P".into()
        )
        .is_err());
    svc.verify_organization(&org_id).unwrap();

    // Wrong currency
    assert!(svc
        .add_employee(
            org_id.clone(),
            "Emp".into(),
            "0xemp".into(),
            100.0,
            "USD".into()
        )
        .is_err());

    // School-only constraints
    let school_id = svc
        .register_organization(
            "SchoolX".into(),
            OrganizationType::School,
            "0xschool".into(),
        )
        .unwrap();
    svc.verify_organization(&school_id).unwrap();
    assert!(svc
        .add_employee(
            school_id.clone(),
            "Emp".into(),
            "0xemp".into(),
            100.0,
            "P".into()
        )
        .is_err());
}
