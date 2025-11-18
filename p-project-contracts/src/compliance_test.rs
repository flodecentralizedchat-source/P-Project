use crate::compliance::{ComplianceError, ComplianceManager};

#[test]
fn compliance_allows_basic_when_unpaused_and_open() {
    let mgr = ComplianceManager::new();
    assert!(mgr.check_transfer("alice", "bob", "token1").is_ok());
}

#[test]
fn compliance_paused_blocks_all() {
    let mut mgr = ComplianceManager::new();
    mgr.set_paused(true);
    let err = mgr
        .check_transfer("alice", "bob", "token1")
        .expect_err("should be paused");
    assert_eq!(err, ComplianceError::Paused);
}

#[test]
fn compliance_blocklist_works() {
    let mut mgr = ComplianceManager::new();
    mgr.block_address("alice");
    let err = mgr
        .check_transfer("alice", "bob", "token1")
        .expect_err("sender blocked");
    assert_eq!(err, ComplianceError::SenderBlocked);

    let mut mgr2 = ComplianceManager::new();
    mgr2.block_address("bob");
    let err2 = mgr2
        .check_transfer("alice", "bob", "token1")
        .expect_err("recipient blocked");
    assert_eq!(err2, ComplianceError::RecipientBlocked);
}

#[test]
fn compliance_token_allowlist_enforced() {
    let mut mgr = ComplianceManager::new();
    mgr.set_enforce_token_allowlist(true);
    mgr.allow_token("token1");

    assert!(mgr.check_transfer("alice", "bob", "token1").is_ok());
    let err = mgr
        .check_transfer("alice", "bob", "token2")
        .expect_err("token2 not allowed");
    assert_eq!(err, ComplianceError::TokenNotAllowed);
}

#[test]
fn compliance_kyc_required_and_enforced() {
    let mut mgr = ComplianceManager::new();
    mgr.set_kyc_required(true);

    // Neither approved
    let err = mgr
        .check_transfer("alice", "bob", "token1")
        .expect_err("kyc required");
    assert_eq!(err, ComplianceError::KycRequired);

    // Only sender approved
    mgr.approve_kyc("alice");
    let err = mgr
        .check_transfer("alice", "bob", "token1")
        .expect_err("recipient missing kyc");
    assert_eq!(err, ComplianceError::KycMissingRecipient);

    // Both approved
    mgr.approve_kyc("bob");
    assert!(mgr.check_transfer("alice", "bob", "token1").is_ok());
}

#[test]
fn compliance_selective_kyc_targets() {
    let mut mgr = ComplianceManager::new();
    mgr.require_selective_kyc("vip");

    let err = mgr
        .check_transfer("vip", "bot", "tokenX")
        .expect_err("selective kyc required");
    assert_eq!(err, ComplianceError::KycRequired);

    mgr.approve_kyc("vip");
    assert!(mgr.check_transfer("vip", "bot", "tokenX").is_ok());

    mgr.clear_selective_kyc("vip");
    assert!(mgr.check_transfer("vip", "bot", "tokenX").is_ok());
}
