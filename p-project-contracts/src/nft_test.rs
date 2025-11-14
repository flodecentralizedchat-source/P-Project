#[cfg(test)]
mod tests {
    use super::super::nft::{NFTContract, NFTMetadata};
    use std::collections::HashMap;

    #[test]
    fn test_create_collection() {
        let mut nft_contract = NFTContract::new();
        
        let collection_id = nft_contract.create_collection(
            "Test Collection".to_string(),
            "TST".to_string(),
            "creator1".to_string(),
            "A test collection".to_string(),
            Some(100),
            true,
        ).unwrap();
        
        assert_eq!(collection_id, "collection_1");
        
        let collection = nft_contract.get_collection(collection_id).unwrap();
        assert_eq!(collection.name, "Test Collection");
        assert_eq!(collection.symbol, "TST");
        assert_eq!(collection.creator, "creator1");
        assert_eq!(collection.description, "A test collection");
        assert_eq!(collection.max_supply, Some(100));
        assert_eq!(collection.is_public, true);
    }

    #[test]
    fn test_mint_nft() {
        let mut nft_contract = NFTContract::new();
        
        // Create a collection first
        let collection_id = nft_contract.create_collection(
            "Test Collection".to_string(),
            "TST".to_string(),
            "creator1".to_string(),
            "A test collection".to_string(),
            Some(100),
            true,
        ).unwrap();
        
        // Create metadata
        let mut attributes = HashMap::new();
        attributes.insert("color".to_string(), "blue".to_string());
        attributes.insert("rarity".to_string(), "common".to_string());
        
        let metadata = NFTMetadata {
            name: "Test NFT".to_string(),
            description: "A test NFT".to_string(),
            image: "ipfs://QmTest123".to_string(),
            attributes,
        };
        
        // Mint NFT
        let nft_id = nft_contract.mint_nft(
            collection_id,
            "owner1".to_string(),
            metadata.clone(),
            5.0, // 5% royalty
        ).unwrap();
        
        assert_eq!(nft_id, "nft_1");
        
        let nft = nft_contract.get_nft(nft_id).unwrap();
        assert_eq!(nft.id, "nft_1");
        assert_eq!(nft.collection_id, "collection_1");
        assert_eq!(nft.owner, "owner1");
        assert_eq!(nft.creator, "creator1");
        assert_eq!(nft.metadata.name, "Test NFT");
        assert_eq!(nft.metadata.description, "A test NFT");
        assert_eq!(nft.metadata.image, "ipfs://QmTest123");
        // Check that the creator is in the royalty recipients with 5% royalty
        assert_eq!(*nft.royalty_recipients.get("creator1").unwrap(), 5.0);
    }

    #[test]
    fn test_transfer_nft() {
        let mut nft_contract = NFTContract::new();
        
        // Create a collection
        let collection_id = nft_contract.create_collection(
            "Test Collection".to_string(),
            "TST".to_string(),
            "creator1".to_string(),
            "A test collection".to_string(),
            Some(100),
            true,
        ).unwrap();
        
        // Create metadata
        let metadata = NFTMetadata {
            name: "Test NFT".to_string(),
            description: "A test NFT".to_string(),
            image: "ipfs://QmTest123".to_string(),
            attributes: HashMap::new(),
        };
        
        // Mint NFT
        let nft_id = nft_contract.mint_nft(
            collection_id,
            "owner1".to_string(),
            metadata,
            5.0,
        ).unwrap();
        
        // Transfer NFT
        let result = nft_contract.transfer_nft(nft_id.clone(), "owner1".to_string(), "owner2".to_string());
        assert!(result.is_ok());
        
        let nft = nft_contract.get_nft(nft_id).unwrap();
        assert_eq!(nft.owner, "owner2");
        assert!(nft.transferred_at.is_some());
    }

    #[test]
    fn test_list_nft() {
        let mut nft_contract = NFTContract::new();
        
        // Create a collection
        let collection_id = nft_contract.create_collection(
            "Test Collection".to_string(),
            "TST".to_string(),
            "creator1".to_string(),
            "A test collection".to_string(),
            Some(100),
            true,
        ).unwrap();
        
        // Create metadata
        let metadata = NFTMetadata {
            name: "Test NFT".to_string(),
            description: "A test NFT".to_string(),
            image: "ipfs://QmTest123".to_string(),
            attributes: HashMap::new(),
        };
        
        // Mint NFT
        let nft_id = nft_contract.mint_nft(
            collection_id,
            "owner1".to_string(),
            metadata,
            5.0,
        ).unwrap();
        
        // List NFT for sale
        let listing_id = nft_contract.list_nft(
            nft_id,
            "owner1".to_string(),
            100.0, // Price
            "P".to_string(), // Currency
            Some(86400), // Expires in 1 day
        ).unwrap();
        
        assert_eq!(listing_id, "listing_1");
        
        let active_listings = nft_contract.get_active_listings();
        assert_eq!(active_listings.len(), 1);
        assert_eq!(active_listings[0].id, "listing_1");
        assert_eq!(active_listings[0].nft_id, "nft_1");
        assert_eq!(active_listings[0].seller, "owner1");
        assert_eq!(active_listings[0].price, 100.0);
        assert_eq!(active_listings[0].currency, "P");
        assert_eq!(active_listings[0].is_active, true);
    }

    #[test]
    fn test_buy_nft() {
        let mut nft_contract = NFTContract::new();
        
        // Create a collection
        let collection_id = nft_contract.create_collection(
            "Test Collection".to_string(),
            "TST".to_string(),
            "creator1".to_string(),
            "A test collection".to_string(),
            Some(100),
            true,
        ).unwrap();
        
        // Create metadata
        let metadata = NFTMetadata {
            name: "Test NFT".to_string(),
            description: "A test NFT".to_string(),
            image: "ipfs://QmTest123".to_string(),
            attributes: HashMap::new(),
        };
        
        // Mint NFT
        let nft_id = nft_contract.mint_nft(
            collection_id,
            "owner1".to_string(),
            metadata,
            10.0, // 10% royalty
        ).unwrap();
        
        // List NFT for sale
        let listing_id = nft_contract.list_nft(
            nft_id.clone(),
            "owner1".to_string(),
            100.0, // Price
            "P".to_string(), // Currency
            Some(86400), // Expires in 1 day
        ).unwrap();
        
        // Set up buyer balance
        nft_contract.owner_balances.insert("buyer1".to_string(), 200.0);
        
        // Buy NFT
        let result = nft_contract.buy_nft(listing_id, "buyer1".to_string(), 100.0);
        assert!(result.is_ok());
        
        // Check ownership transferred
        let nft = nft_contract.get_nft(nft_id).unwrap();
        assert_eq!(nft.owner, "buyer1");
        
        // Check balances updated
        let buyer_balance = nft_contract.get_user_earnings("buyer1".to_string());
        assert_eq!(buyer_balance, 100.0); // 200 - 100
        
        let seller_balance = nft_contract.get_user_earnings("owner1".to_string());
        assert_eq!(seller_balance, 90.0); // 100 - 10% royalty
        
        let creator_royalties = nft_contract.get_user_royalties("creator1".to_string());
        // Debug: print the actual value
        println!("Creator royalties: {}", creator_royalties);
        assert_eq!(creator_royalties, 10.0); // 10% of 100
    }

    #[test]
    fn test_withdraw_earnings() {
        let mut nft_contract = NFTContract::new();
        
        // Set up user balance
        nft_contract.owner_balances.insert("user1".to_string(), 150.0);
        
        // Withdraw earnings
        let remaining_balance = nft_contract.withdraw_earnings("user1".to_string(), 50.0).unwrap();
        assert_eq!(remaining_balance, 100.0);
        
        let user_balance = nft_contract.get_user_earnings("user1".to_string());
        assert_eq!(user_balance, 100.0);
    }

    #[test]
    fn test_get_user_nfts() {
        let mut nft_contract = NFTContract::new();
        
        // Create a collection
        let collection_id = nft_contract.create_collection(
            "Test Collection".to_string(),
            "TST".to_string(),
            "creator1".to_string(),
            "A test collection".to_string(),
            Some(100),
            true,
        ).unwrap();
        
        // Create metadata
        let metadata1 = NFTMetadata {
            name: "Test NFT 1".to_string(),
            description: "A test NFT".to_string(),
            image: "ipfs://QmTest123".to_string(),
            attributes: HashMap::new(),
        };
        
        let metadata2 = NFTMetadata {
            name: "Test NFT 2".to_string(),
            description: "Another test NFT".to_string(),
            image: "ipfs://QmTest456".to_string(),
            attributes: HashMap::new(),
        };
        
        // Mint NFTs
        nft_contract.mint_nft(
            collection_id.clone(),
            "owner1".to_string(),
            metadata1,
            5.0,
        ).unwrap();
        
        nft_contract.mint_nft(
            collection_id,
            "owner1".to_string(),
            metadata2,
            5.0,
        ).unwrap();
        
        // Get user's NFTs
        let user_nfts = nft_contract.get_user_nfts("owner1".to_string());
        assert_eq!(user_nfts.len(), 2);
    }

    #[test]
    fn test_collection_supply_limit() {
        let mut nft_contract = NFTContract::new();
        
        // Create a collection with max supply of 1
        let collection_id = nft_contract.create_collection(
            "Limited Collection".to_string(),
            "LTD".to_string(),
            "creator1".to_string(),
            "A limited collection".to_string(),
            Some(1),
            true,
        ).unwrap();
        
        // Create metadata
        let metadata = NFTMetadata {
            name: "Test NFT".to_string(),
            description: "A test NFT".to_string(),
            image: "ipfs://QmTest123".to_string(),
            attributes: HashMap::new(),
        };
        
        // Mint first NFT (should succeed)
        let result1 = nft_contract.mint_nft(
            collection_id.clone(),
            "owner1".to_string(),
            metadata.clone(),
            5.0,
        );
        assert!(result1.is_ok());
        
        // Try to mint second NFT (should fail)
        let result2 = nft_contract.mint_nft(
            collection_id,
            "owner2".to_string(),
            metadata,
            5.0,
        );
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), "Collection has reached maximum supply");
    }

    #[test]
    fn test_cancel_listing() {
        let mut nft_contract = NFTContract::new();
        
        // Create a collection
        let collection_id = nft_contract.create_collection(
            "Test Collection".to_string(),
            "TST".to_string(),
            "creator1".to_string(),
            "A test collection".to_string(),
            Some(100),
            true,
        ).unwrap();
        
        // Create metadata
        let metadata = NFTMetadata {
            name: "Test NFT".to_string(),
            description: "A test NFT".to_string(),
            image: "ipfs://QmTest123".to_string(),
            attributes: HashMap::new(),
        };
        
        // Mint NFT
        let nft_id = nft_contract.mint_nft(
            collection_id,
            "owner1".to_string(),
            metadata,
            5.0,
        ).unwrap();
        
        // List NFT for sale
        let listing_id = nft_contract.list_nft(
            nft_id,
            "owner1".to_string(),
            100.0,
            "P".to_string(),
            Some(86400),
        ).unwrap();
        
        // Cancel listing
        let result = nft_contract.cancel_listing(listing_id.clone(), "owner1".to_string());
        assert!(result.is_ok());
        
        // Check listing is inactive
        let listing = nft_contract.listings.get(&listing_id).unwrap();
        assert_eq!(listing.is_active, false);
    }
}