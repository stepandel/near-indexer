use serde_json::value::Value;
use tracing::{ info, warn };
use near_indexer::near_primitives::types::AccountId;
use serde::{Deserialize, Serialize};
use futures::try_join;

use crate::events::{ Nep171EventKind, Nep141EventKind, NearEvent, NftMintData, NftTransferData, NftBurnData, FtTransferData };
use crate::db_adapters;
use crate::gg_adapters;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Token {
    pub contract_id: AccountId,
    pub token_id: String,
    pub metadata: Option<TokenMetadata>,
}

// media_hash & reference_hash must be Base64VecU8 instead of String
// copies must be U64

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<String>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<String>, // number of copies of this set of metadata in existence when token was minted.
    pub rarity : Option <String>,
    pub nft_type : Option<String>,
    pub collection_name : Option<String>,
    pub issued_at: Option<String>, // ISO 8601 datetime when token was issued or minted
    pub expires_at: Option<String>, // ISO 8601 datetime when token expires
    pub starts_at: Option<String>, // ISO 8601 datetime when token starts being valid
    pub updated_at: Option<String>, // ISO 8601 datetime when token was last updated
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<String>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
    pub game_id: Option<String>,                //the game id
}


impl TokenMetadata {
    fn from_json(json: Option<&serde_json::value::Value>) -> Option<Self> {
        if let Some(json) = json {
            let metadata: TokenMetadata = serde_json::from_value(json.clone()).unwrap();
            return Some(metadata);
        }
        None
    }
}


#[derive(Debug)]
pub struct ArgsJsonTokenMint<'a> {
    pub token_metadata: Option<&'a serde_json::value::Value>,
}


pub(crate) async fn process_token_event(
    pool: &mongodb::Client,
    contract_id: &AccountId,
    receipt_args: &serde_json::Value,
    events: &Vec<NearEvent>,
) {

    for event in events {
        match event {
            NearEvent::Nep171(nep171event) => {
                let event_kind = &nep171event.event_kind;
                match event_kind {
                    Nep171EventKind::NftMint(mints) => process_token_mint(pool, contract_id, receipt_args, mints).await,
                    Nep171EventKind::NftTransfer(transfers) => process_token_transfer(pool, contract_id, transfers).await,
                    Nep171EventKind::NftBurn(burns) => process_token_burn(pool, contract_id, burns).await,
                }
            },
            NearEvent::Nep141(nep141event) => {
                let event_kind = &nep141event.event_kind;
                match event_kind {
                    Nep141EventKind::FtTransfer(transfers) => process_ft_transfer(transfers).await,
                    _ => (),
                }
            },
        }
        
    }
}


pub fn get_token_args(args: &serde_json::Value) -> Vec<ArgsJsonTokenMint> {

    let mut tokens: Vec<ArgsJsonTokenMint> = Vec::new();

    match args {
        serde_json::Value::Object(object_map) => {

            if let Some(data) = object_map.get("args_json") {
                match data {
                    serde_json::Value::Object(object_map) => {
    
                        info!(
                            target: crate::INDEXER,
                            "Object: {:#?}",
                            &object_map,
                        );

                        if let Some(data) = object_map.get("tokens_to_mint") {
                            match data {
                                serde_json::Value::Array(vector) => {
                                    for token_to_mint in vector {

                                        let token_args = ArgsJsonTokenMint {
                                            token_metadata: token_to_mint.get("metadata"),
                                        };

                                        info!(
                                            target: crate::INDEXER,
                                            "Unwrapped token args: {:#?}",
                                            &token_args,
                                        );
                        
                                        tokens.push(token_args);
                                    }
                                },
                                _ => (),
                            }
                        }
              
                    },
                    _ => (),
                }
            }
            
        },
        _ => (),
    }

    tokens
}


pub(super) async fn process_token_mint(
    pool: &mongodb::Client,
    contract_id: &AccountId,
    receipt_args: &serde_json::Value,
    mints: &Vec<NftMintData>,
) {

    let token_args = get_token_args(receipt_args);

    for mint in mints {

        let owner_id = &mint.owner_id;

        info!(
            target: crate::INDEXER,
            "Owner ID: {}",
            &owner_id,
        );

        let token_ids = &mint.token_ids;

        for (i, token_id) in token_ids.iter().enumerate() {

            info!(
                target: crate::INDEXER,
                "Token ID: {}",
                &token_id,
            );

            info!(
                target: crate::INDEXER,
                "Token args: {:#?}",
                &token_args,
            );

            let mut token = Token {
                contract_id: contract_id.clone(),
                token_id: token_id.clone(),
                metadata: None
            };

            if let Some(args) = token_args.get(i) {
                token.metadata = TokenMetadata::from_json(args.token_metadata)
            }

            info!(
                target: crate::INDEXER,
                "Minted Token: {:#?}",
                &token,
            );

            info!(
                target: crate::INDEXER,
                "Minted Token: {:#?}",
                &token,
            );

            match db_adapters::tokens::store_token(&pool, token.clone()).await {
                Ok(_) => {
                    match db_adapters::token_owners::add_token_owner(&pool, contract_id.clone(), token_id.clone(), owner_id.clone()).await {
                        Err(error) => warn!( target: crate::INDEXER, "Error adding token owner to database: {:?}", &error ),
                        _ => (),
                    }
                },
                Err(error) => warn!( target: crate::INDEXER, "Error adding token to database: {:?}", &error ),
            }

            match gg_adapters::mint_game_asset(contract_id.clone(), token_id.clone()).await {
                Err(error) => warn!( target: crate::INDEXER, "Error! Coudn't notify server: {:?}", &error),
                _ => (),
            }
            
        }
    }
}


pub(super) async fn process_token_transfer(
    pool: &mongodb::Client,
    contract_id: &AccountId,
    transfers: &Vec<NftTransferData>,
) {
    for transfer in transfers {

        let old_owner_id = &transfer.old_owner_id;
        let new_owner_id = &transfer.new_owner_id;

        info!{
            target: crate::INDEXER,
            "Transfer from {} to {}",
            &old_owner_id, &new_owner_id,
        };

        let token_ids = &transfer.token_ids;

        for token_id in token_ids {

            info!(
                target: crate::INDEXER,
                "Token ID: {}",
                &token_id,
            );

            let add_new_owner_future = db_adapters::token_owners::add_token_owner(&pool, contract_id.clone(), token_id.clone(), new_owner_id.clone());
            let remove_old_owner_future = db_adapters::token_owners::remove_token_owner(&pool, contract_id.clone(), token_id.clone(), old_owner_id.clone());

            match try_join!(add_new_owner_future, remove_old_owner_future) {
                Err(error) => {
                    warn!(
                        target: crate::INDEXER,
                        "Error writing to database: {:?}",
                        &error,
                    )
                },
                _ => (),
            }
        }
    }
}


pub(super) async fn process_token_burn(
    pool: &mongodb::Client,
    contract_id: &AccountId,
    burns: &Vec<NftBurnData>,
) {
    for burn in burns {

        let token_ids = &burn.token_ids;

        info!(
            target: crate::INDEXER,
            "Burn token_ids: {:#?}",
            &token_ids,
        )
    }
}


pub(super) async fn process_ft_transfer(
    transfers: &Vec<FtTransferData>,
) {
    for transfer in transfers {

        let from_wallet_id = &transfer.old_owner_id;
        let to_wallet_id = &transfer.new_owner_id;
        let amount = &transfer.amount;
        let memo = &transfer.memo;

        info!{
            target: crate::INDEXER,
            "Transfer {} from {} to {}",
            &amount, &from_wallet_id, &from_wallet_id,
        };

        if let Some(memo) = memo {
            info!{
                target: crate::INDEXER,
                "Voucher: {}",
                &memo,
            };
        }

        match gg_adapters::transfer_ft(from_wallet_id.to_string(), to_wallet_id.to_string(), amount.to_string(), memo.to_owned()).await {
            Err(error) => warn!( target: crate::INDEXER, "Error! Coudn't notify server: {:?}", &error),
            _ => (),
        }
    }
}