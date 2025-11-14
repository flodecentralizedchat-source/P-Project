use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};

#[derive(Debug, Clone)]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub image: String, // IPFS CID or URL
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct NFT {
    pub id: String,
    pub collection_id: String,
    pub owner: String,
    pub creator: String,
    pub metadata: NFTMetadata,
    pub royalty_recipients: HashMap<String, f64>, // recipient_address -> percentage
    pub created_at: NaiveDateTime,
    pub transferred_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct NFTCollection {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub creator: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub max_supply: Option<u64>, // None for unlimited
    pub is_public: bool, // Whether anyone can mint or only creator
}

#[derive(Debug, Clone)]
pub struct MarketplaceListing {
    pub id: String,
    pub nft_id: String,
    pub seller: String,
    pub price: f64,
    pub currency: String, // Token symbol (e.g., "P")
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub is_active: bool,
}

pub struct NFTContract {
    pub nfts: HashMap<String, NFT>,
    pub collections: HashMap<String, NFTCollection>,
    pub listings: HashMap<String, MarketplaceListing>,
    pub owner_balances: HashMap<String, f64>, // User address -> token balance for sales
    pub royalty_balances: HashMap<String, f64>, // Recipient address -> accumulated royalties
    pub total_nfts: u64,
    pub total_collections: u64,
    pub total_listings: u64,
}

impl NFTContract {
    pub fn new() -> Self {
        Self {
            nfts: HashMap::new(),
            collections: HashMap::new(),
            listings: HashMap::new(),
            owner_balances: HashMap::new(),
            royalty_balances: HashMap::new(),
            total_nfts: 0,
            total_collections: 0,
            total_listings: 0,
        }
    }

    /// Create a new NFT collection
    pub fn create_collection(
        &mut self,
        name: String,
        symbol: String,
        creator: String,
        description: String,
        max_supply: Option<u64>,
        is_public: bool,
    ) -> Result<String, String> {
        let collection_id = format!("collection_{}", self.total_collections + 1);
        
        let collection = NFTCollection {
            id: collection_id.clone(),
            name,
            symbol,
            creator: creator.clone(),
            description,
            created_at: Utc::now().naive_utc(),
            max_supply,
            is_public,
        };
        
        self.collections.insert(collection_id.clone(), collection);
        self.total_collections += 1;
        
        Ok(collection_id)
    }

    /// Mint a new NFT in a collection
    pub fn mint_nft(
        &mut self,
        collection_id: String,
        owner: String,
        metadata: NFTMetadata,
        royalty_percentage: f64,
    ) -> Result<String, String> {
        // Check if collection exists
        let collection = self.collections.get(&collection_id).ok_or("Collection not found")?.clone();
        
        // Check if caller can mint (creator or public collection)
        // In a real implementation, we would verify the caller's identity
        // For now, we'll assume the caller has permission
        
        // Check collection supply limit
        if let Some(max_supply) = collection.max_supply {
            let collection_nft_count = self.nfts.values()
                .filter(|nft| nft.collection_id == collection_id)
                .count() as u64;
                
            if collection_nft_count >= max_supply {
                return Err("Collection has reached maximum supply".to_string());
            }
        }
        
        // Validate royalty percentage
        if royalty_percentage < 0.0 || royalty_percentage > 50.0 {
            return Err("Royalty percentage must be between 0 and 50".to_string());
        }
        
        let nft_id = format!("nft_{}", self.total_nfts + 1);
        
        // Initialize royalty recipients with creator getting the full percentage
        let mut royalty_recipients = HashMap::new();
        royalty_recipients.insert(collection.creator.clone(), royalty_percentage);
        
        let nft = NFT {
            id: nft_id.clone(),
            collection_id: collection_id.clone(),
            owner: owner.clone(),
            creator: collection.creator.clone(),
            metadata,
            royalty_recipients,
            created_at: Utc::now().naive_utc(),
            transferred_at: None,
        };
        
        self.nfts.insert(nft_id.clone(), nft);
        self.total_nfts += 1;
        
        Ok(nft_id)
    }

    /// Transfer NFT ownership
    pub fn transfer_nft(&mut self, nft_id: String, from: String, to: String) -> Result<(), String> {
        let nft = self.nfts.get_mut(&nft_id).ok_or("NFT not found")?;
        
        // Verify ownership
        if nft.owner != from {
            return Err("Not the owner of this NFT".to_string());
        }
        
        // Update ownership
        nft.owner = to;
        nft.transferred_at = Some(Utc::now().naive_utc());
        
        Ok(())
    }

    /// List NFT for sale on marketplace
    pub fn list_nft(
        &mut self,
        nft_id: String,
        seller: String,
        price: f64,
        currency: String,
        expires_in_seconds: Option<i64>,
    ) -> Result<String, String> {
        let nft = self.nfts.get(&nft_id).ok_or("NFT not found")?;
        
        // Verify ownership
        if nft.owner != seller {
            return Err("Not the owner of this NFT".to_string());
        }
        
        // Check if NFT is already listed
        for listing in self.listings.values() {
            if listing.nft_id == nft_id && listing.is_active {
                return Err("NFT is already listed for sale".to_string());
            }
        }
        
        let listing_id = format!("listing_{}", self.total_listings + 1);
        
        let expires_at = expires_in_seconds.map(|seconds| {
            Utc::now().naive_utc() + chrono::Duration::seconds(seconds)
        });
        
        let listing = MarketplaceListing {
            id: listing_id.clone(),
            nft_id: nft_id.clone(),
            seller: seller.clone(),
            price,
            currency,
            created_at: Utc::now().naive_utc(),
            expires_at,
            is_active: true,
        };
        
        self.listings.insert(listing_id.clone(), listing);
        self.total_listings += 1;
        
        Ok(listing_id)
    }

    /// Cancel NFT listing
    pub fn cancel_listing(&mut self, listing_id: String, seller: String) -> Result<(), String> {
        let listing = self.listings.get_mut(&listing_id).ok_or("Listing not found")?;
        
        // Verify ownership
        if listing.seller != seller {
            return Err("Not the seller of this listing".to_string());
        }
        
        // Mark as inactive
        listing.is_active = false;
        
        Ok(())
    }

    /// Buy NFT from marketplace
    pub fn buy_nft(
        &mut self,
        listing_id: String,
        buyer: String,
        payment_amount: f64,
    ) -> Result<(), String> {
        // Clone necessary data to avoid borrowing issues
        let (nft_id, seller, price) = {
            let listing = self.listings.get(&listing_id).ok_or("Listing not found")?;
            
            // Check if listing is active
            if !listing.is_active {
                return Err("Listing is not active".to_string());
            }
            
            // Check if listing has expired
            if let Some(expires_at) = listing.expires_at {
                if Utc::now().naive_utc() > expires_at {
                    // We can't modify the listing here, so we'll handle this in the next step
                    return Err("Listing has expired".to_string());
                }
            }
            
            // Check payment amount
            if payment_amount < listing.price {
                return Err("Insufficient payment amount".to_string());
            }
            
            (listing.nft_id.clone(), listing.seller.clone(), listing.price)
        };
        
        // Check if listing has expired (second check to update the listing)
        let expired = {
            let listing = self.listings.get(&listing_id).unwrap();
            if let Some(expires_at) = listing.expires_at {
                Utc::now().naive_utc() > expires_at
            } else {
                false
            }
        };
        
        if expired {
            let listing = self.listings.get_mut(&listing_id).unwrap();
            listing.is_active = false;
            return Err("Listing has expired".to_string());
        }
        
        // Transfer NFT ownership
        self.transfer_nft(nft_id.clone(), seller.clone(), buyer.clone())?;
        
        // Process payment
        // Calculate total royalty amount from all recipients
        let nft = self.nfts.get(&nft_id).ok_or("NFT not found")?;
        let total_royalty_percentage: f64 = nft.royalty_recipients.values().sum();
        let royalty_amount = price * (total_royalty_percentage / 100.0);
        let seller_amount = price - royalty_amount;
        
        // Update balances
        *self.owner_balances.entry(buyer).or_insert(0.0) -= price;
        *self.owner_balances.entry(seller).or_insert(0.0) += seller_amount;
        
        // Distribute royalties to all recipients
        if royalty_amount > 0.0 {
            let nft = self.nfts.get(&nft_id).ok_or("NFT not found")?;
            for (recipient, percentage) in &nft.royalty_recipients {
                let recipient_amount = royalty_amount * (percentage / total_royalty_percentage);
                *self.royalty_balances.entry(recipient.clone()).or_insert(0.0) += recipient_amount;
            }
        }
        
        // Mark listing as inactive
        let listing = self.listings.get_mut(&listing_id).unwrap();
        listing.is_active = false;
        
        Ok(())
    }

    /// Withdraw earnings from sales
    pub fn withdraw_earnings(&mut self, user: String, amount: f64) -> Result<f64, String> {
        let balance = self.owner_balances.get_mut(&user).ok_or("No balance found")?;
        
        if *balance < amount {
            return Err("Insufficient balance".to_string());
        }
        
        *balance -= amount;
        Ok(*balance)
    }

    /// Withdraw accumulated royalties
    pub fn withdraw_royalties(&mut self, recipient: String, amount: f64) -> Result<f64, String> {
        let balance = self.royalty_balances.get_mut(&recipient).ok_or("No royalty balance found")?;
        
        if *balance < amount {
            return Err("Insufficient royalty balance".to_string());
        }
        
        *balance -= amount;
        Ok(*balance)
    }

    /// Add a royalty recipient to an NFT
    pub fn add_royalty_recipient(&mut self, nft_id: String, recipient: String, percentage: f64) -> Result<(), String> {
        let nft = self.nfts.get_mut(&nft_id).ok_or("NFT not found")?;
        
        // Validate percentage
        if percentage <= 0.0 || percentage > 100.0 {
            return Err("Royalty percentage must be between 0 and 100".to_string());
        }
        
        // Check total royalty percentage doesn't exceed 100%
        let current_total: f64 = nft.royalty_recipients.values().sum();
        if current_total + percentage > 100.0 {
            return Err("Total royalty percentage cannot exceed 100%".to_string());
        }
        
        // Add recipient
        nft.royalty_recipients.insert(recipient, percentage);
        Ok(())
    }

    /// Remove a royalty recipient from an NFT
    pub fn remove_royalty_recipient(&mut self, nft_id: String, recipient: String) -> Result<f64, String> {
        let nft = self.nfts.get_mut(&nft_id).ok_or("NFT not found")?;
        
        // Cannot remove the original creator
        if nft.creator == recipient {
            return Err("Cannot remove the original creator from royalty recipients".to_string());
        }
        
        // Remove recipient and return their percentage
        nft.royalty_recipients.remove(&recipient).ok_or("Recipient not found in royalty distribution".to_string())
    }

    /// Get royalty recipients for an NFT
    pub fn get_royalty_recipients(&self, nft_id: String) -> Result<&HashMap<String, f64>, String> {
        let nft = self.nfts.get(&nft_id).ok_or("NFT not found")?;
        Ok(&nft.royalty_recipients)
    }

    /// Get NFT by ID
    pub fn get_nft(&self, nft_id: String) -> Option<&NFT> {
        self.nfts.get(&nft_id)
    }

    /// Get collection by ID
    pub fn get_collection(&self, collection_id: String) -> Option<&NFTCollection> {
        self.collections.get(&collection_id)
    }

    /// Get active listings
    pub fn get_active_listings(&self) -> Vec<&MarketplaceListing> {
        self.listings.values()
            .filter(|listing| listing.is_active)
            .filter(|listing| {
                if let Some(expires_at) = listing.expires_at {
                    Utc::now().naive_utc() <= expires_at
                } else {
                    true
                }
            })
            .collect()
    }

    /// Get user's NFTs
    pub fn get_user_nfts(&self, user: String) -> Vec<&NFT> {
        self.nfts.values()
            .filter(|nft| nft.owner == user)
            .collect()
    }

    /// Get collection NFTs
    pub fn get_collection_nfts(&self, collection_id: String) -> Vec<&NFT> {
        self.nfts.values()
            .filter(|nft| nft.collection_id == collection_id)
            .collect()
    }

    /// Get user's earnings
    pub fn get_user_earnings(&self, user: String) -> f64 {
        *self.owner_balances.get(&user).unwrap_or(&0.0)
    }

    /// Get user's accumulated royalties
    pub fn get_user_royalties(&self, user: String) -> f64 {
        *self.royalty_balances.get(&user).unwrap_or(&0.0)
    }
}