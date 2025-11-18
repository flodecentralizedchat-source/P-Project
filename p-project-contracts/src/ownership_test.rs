use crate::ownership::{Ownable, OwnershipError};

#[test]
fn ownership_transfer_and_renounce() {
    let mut ownable = Ownable::new("admin");
    assert_eq!(ownable.owner(), Some("admin"));

    // Non-owner cannot transfer
    let err = ownable
        .transfer_ownership("eve", "bob")
        .expect_err("non-owner should fail");
    assert_eq!(err, OwnershipError::NotOwner);

    // Owner transfers
    ownable.transfer_ownership("admin", "bob").unwrap();
    assert_eq!(ownable.owner(), Some("bob"));

    // New owner renounces
    ownable.renounce_ownership("bob").unwrap();
    assert!(ownable.is_renounced());

    // After renounce, privileged actions fail with AlreadyRenounced
    let err = ownable
        .transfer_ownership("bob", "carol")
        .expect_err("renounced");
    assert_eq!(err, OwnershipError::AlreadyRenounced);
}
