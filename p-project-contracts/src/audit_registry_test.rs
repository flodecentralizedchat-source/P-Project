use crate::audit_registry::{AuditError, AuditInfo, AuditRegistry};

#[test]
fn audit_set_and_get() {
    let mut reg = AuditRegistry::new();
    let info = AuditInfo::new(
        "Trail of Bits",
        "ipfs://QmAuditReport",
        "0xdeadbeef",
        Some("Initial audit"),
    );
    reg.set_audit("Bridge", info.clone()).unwrap();

    let fetched = reg.get("Bridge").expect("audit exists");
    assert_eq!(fetched.firm, "Trail of Bits");
    assert!(!fetched.finalized);
}

#[test]
fn audit_finalize_prevents_updates() {
    let mut reg = AuditRegistry::new();
    let info = AuditInfo::new("OpenZeppelin", "https://example.com/r.pdf", "0x1234", None);
    reg.set_audit("Token", info).unwrap();
    reg.finalize("Token").unwrap();
    assert!(reg.is_finalized("Token").unwrap());

    let new_info = AuditInfo::new("OtherFirm", "https://new", "0xabcd", None);
    let err = reg
        .set_audit("Token", new_info)
        .expect_err("should be finalized");
    assert_eq!(err, AuditError::AlreadyFinalized);
}
