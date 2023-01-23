use mongodb::bson::doc;
use mongodb::options::UpdateOptions;
use near_indexer::near_primitives::types::AccountId;
use serde::{ Deserialize, Serialize };

use tracing::info;
use crate::utils;
use super::tokens::TokenDB;


#[derive(Debug, Serialize, Deserialize)]
pub struct NearWalletTokensDB {
    _id: String,
    pub near_wallet: AccountId,
    pub tokens: Vec<String>,
}

pub(crate) async fn add_token_owner(
    pool: &mongodb::Client,
    contract_id: AccountId,
    token_id: String,
    new_owner: String,
) -> anyhow::Result<()> {

    info!(
        target: crate::INDEXER,
        "Updating new owner {} for token {} in contract {}",
        &new_owner, &token_id, &contract_id
    );

    let db = pool.database(crate::DB_NAME);

    let token_id_hash = utils::keccak256_hash_string(format!("{}{}", contract_id, token_id));

    {
        let wallet_tokens_collection = db.collection::<NearWalletTokensDB>(super::WALLET_TOKENS);

        let query = doc!{ "_id": new_owner.clone() };
        let update = doc!{ "$push": { "tokens": token_id_hash.clone() } };
        let options = UpdateOptions::builder().upsert(true).build();

        crate::await_retry_or_panic!(
            wallet_tokens_collection.update_one(query.clone(), update.clone(), options.clone()),
            10,
            "New owner was NOT updated in database".to_string(),
            (&token_id, &new_owner),
        );
    }

    // Update chain-nfts table to add to token owenership history
    {
        let token_collection = db.collection::<TokenDB>(super::TOKEN_TABLE);

        let query = doc!{ "_id": token_id_hash };
        let update = doc!{ "$push": { "ownership_history": new_owner.clone() }};
        let options = UpdateOptions::builder().upsert(true).build();

        crate::await_retry_or_panic!(
            token_collection.update_one(query.clone(), update.clone(), options.clone()),
            10,
            "Ownership history was not updated in database".to_string(),
            (&token_id, &new_owner),
        );
    }

    Ok(())
}

pub(crate) async fn remove_token_owner(
    pool: &mongodb::Client,
    contract_id: AccountId,
    token_id: String,
    old_owner: String,
) -> anyhow::Result<()> {

    info!(
        target: crate::INDEXER,
        "Removing owner {} for token {} in contract {}",
        &old_owner, &token_id, &contract_id,
    );

    let db = pool.database(crate::DB_NAME);
    let wallet_tokens_collection = db.collection::<NearWalletTokensDB>(super::WALLET_TOKENS);

    let token_id_hash = utils::keccak256_hash_string(format!("{}{}", contract_id, token_id));

    let query = doc!{ "_id": old_owner.clone() };
    let update = doc!{ "$pull": { "tokens": token_id_hash } };
    let options = UpdateOptions::builder().upsert(true).build();

    crate::await_retry_or_panic!(
        wallet_tokens_collection.update_one(query.clone(), update.clone(), options.clone()),
        10,
        "Old owner was NOT removed from database".to_string(),
        (&token_id, &old_owner),
    );

    Ok(())
}