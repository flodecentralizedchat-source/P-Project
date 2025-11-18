use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipError {
    NotOwner,
    AlreadyRenounced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ownable {
    owner: Option<String>,
}

impl Ownable {
    pub fn new<S: Into<String>>(owner: S) -> Self {
        Self {
            owner: Some(owner.into()),
        }
    }

    pub fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub fn is_renounced(&self) -> bool {
        self.owner.is_none()
    }

    pub fn ensure_owner(&self, caller: &str) -> Result<(), OwnershipError> {
        match &self.owner {
            Some(o) if o == caller => Ok(()),
            Some(_) => Err(OwnershipError::NotOwner),
            None => Err(OwnershipError::AlreadyRenounced),
        }
    }

    pub fn transfer_ownership(
        &mut self,
        caller: &str,
        new_owner: &str,
    ) -> Result<(), OwnershipError> {
        self.ensure_owner(caller)?;
        self.owner = Some(new_owner.to_string());
        Ok(())
    }

    pub fn renounce_ownership(&mut self, caller: &str) -> Result<(), OwnershipError> {
        self.ensure_owner(caller)?;
        self.owner = None;
        Ok(())
    }
}
