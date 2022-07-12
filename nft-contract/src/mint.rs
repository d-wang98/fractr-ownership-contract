use crate::*;
// use std::collections::HashSet;
use near_sdk::collections::{UnorderedSet};

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn art_mint(
        &mut self,
        art_id: ArtId,
        art_title: String,
        artist_name: String,
        authenticator: String,
        num_shares: u64,
        minter_id: AccountId,
        


    //     pub art_title: String, // name of the artwork e.g. Mona Lisa
    //     pub artist_name: String, // name of the artist
    //     pub authenticator: String, // person who authenticated the artwork. 
    //     pub issued_at: u64, // date at which the artwork was issued on the fractr platform
    //     pub num_of_shares: u64, // number of shares that have been issued
    ) {
        let initial_storage_usage = env::storage_usage();
        
        let art = Art {
            art_title,
            artist_name,
            authenticator,
            num_shares,
        };

        assert!(
            self.art_by_id.insert(&art_id, &art).is_none(),
            "Art already exists"
        );

        let mut temp_set: UnorderedSet<TokenId> = UnorderedSet::new(b"s"); 
        
        // mint num_share tokens
        for i in 0..num_shares {

            let token = Token {
                //set the owner ID equal to the receiver ID passed into the function
                owner_id: minter_id.clone(),
                //we set the approved account IDs to the default value (an empty map)
                approved_account_ids: Default::default(),
                //the next approval ID is set to 0
                next_approval_id: 0,
                //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
                royalty: HashMap::new(),
            };

            // generate token ID
            // TODO: Make the token id not jank
            let curr_token_id = art_id.clone().to_string() + &i.to_string();

            assert!(
                self.tokens_by_id.insert(&curr_token_id, &token).is_none(),
                "Token already exists"
            );
            
            // insert sample metadata into token metadata map
            self.token_metadata_by_id.insert(
                &curr_token_id, 
                &TokenMetadata {
                    title: Some("Olympus Mons".into()),
                    description: Some("The tallest mountain in the charted solar system".into()),
                    media: None,
                    media_hash: None,
                    copies: Some(1u64),
                    issued_at: None,
                    expires_at: None,
                    starts_at: None,
                    updated_at: None,
                    extra: None,
                    reference: None,
                    reference_hash: None,
                }
            );
            
            // call the internal method for adding the token to the owner
            self.internal_add_token_to_owner(&token.owner_id, &curr_token_id);

            // add token to set of tokens for the art we are currently minting
            temp_set.insert(&curr_token_id);
            
            let nft_mint_log: EventLog = EventLog {
                // Standard name ("nep171").
                standard: NFT_STANDARD_NAME.to_string(),
                // Version of the standard ("nft-1.0.0").
                version: NFT_METADATA_SPEC.to_string(),
                // The data related with the event stored in a vector.
                event: EventLogVariant::NftMint(vec![NftMintLog {
                    // Owner of the token.
                    owner_id: token.owner_id.to_string(),
                    // Vector of token IDs that were minted.
                    token_ids: vec![curr_token_id.to_string()],
                    // An optional memo to include.
                    memo: None,
                }]),
            };

            env::log_str(&nft_mint_log.to_string());
        }

        // map art id to the set of tokens minted
        self.tokens_per_art.insert(&art_id, &temp_set);
        
        // calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);

    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        //we add an optional parameter for perpetual royalties
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) {
        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        // create a royalty map to store in the token
        let mut royalty = HashMap::new();

        // if perpetual royalties were passed into the function: 
        if let Some(perpetual_royalties) = perpetual_royalties {
            //make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

            //iterate through the perpetual royalties and insert the account and amount in the royalty map
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        //specify the token struct that contains the owner ID 
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
            //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
            royalty,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exists"
        );

        //insert the token ID and metadata
        self.token_metadata_by_id.insert(&token_id, &metadata);

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // Owner of the token.
                owner_id: token.owner_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);
    }
}