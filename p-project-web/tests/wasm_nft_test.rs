#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    use p_project_web::wasm_components::*;

    #[wasm_bindgen_test]
    fn test_create_nft_collection() {
        let collection = create_nft_collection(
            "Test Collection",
            "TST",
            "creator1",
            "A test collection"
        );
        
        assert_eq!(collection.name(), "Test Collection");
        assert_eq!(collection.symbol(), "TST");
        assert_eq!(collection.creator(), "creator1");
        assert_eq!(collection.description(), "A test collection");
    }

    #[wasm_bindgen_test]
    fn test_mint_nft() {
        let nft = mint_nft(
            "collection-1",
            "Test NFT",
            "A test NFT",
            "ipfs://QmTest123",
            "owner1",
            5.0
        );
        
        assert_eq!(nft.name(), "Test NFT");
        assert_eq!(nft.description(), "A test NFT");
        assert_eq!(nft.image(), "ipfs://QmTest123");
        assert_eq!(nft.owner(), "owner1");
        assert_eq!(nft.creator(), "owner1");
        assert_eq!(nft.royalty_percentage(), 5.0);
    }

    #[wasm_bindgen_test]
    fn test_list_nft_for_sale() {
        let listing = list_nft_for_sale(
            "nft-1",
            "seller1",
            100.0,
            "P"
        );
        
        assert_eq!(listing.nft_id(), "nft-1");
        assert_eq!(listing.seller(), "seller1");
        assert_eq!(listing.price(), 100.0);
        assert_eq!(listing.currency(), "P");
    }

    #[wasm_bindgen_test]
    fn test_buy_nft() {
        let result = buy_nft("listing-1", "buyer1");
        assert_eq!(result, true);
    }

    #[wasm_bindgen_test]
    fn test_get_user_nfts() {
        let result = get_user_nfts("user1");
        assert_eq!(result, "[]");
    }

    #[wasm_bindgen_test]
    fn test_get_active_listings() {
        let result = get_active_listings();
        assert_eq!(result, "[]");
    }
}