use mongodb::bson::doc;
use near_indexer::near_primitives::types::AccountId;
use serde::{ Deserialize, Serialize };

use tracing::info;
use crate::models::token::{ Token, TokenMetadata };
use crate::utils;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TokenDB {
    _id: String,
    pub contract_id: AccountId,
    pub token_id: String,
    pub metadata: Option<TokenMetadata>,
}

pub(crate) async fn store_token(
    pool: &mongodb::Client,
    token: Token,
) -> anyhow::Result<()> {

    let id_hash = utils::keccak256_hash_string(format!("{}{}", token.contract_id, token.token_id));

    let token_db = TokenDB {
        _id: id_hash,
        contract_id: token.contract_id,
        token_id: token.token_id,
        metadata: token.metadata,
    };

    info!(
        target: crate::INDEXER,
        "Adding new token to DB: {:#?}",
        &token_db,
    );

    let db = pool.database(crate::DB_NAME);
    let token_collection = db.collection::<TokenDB>(super::TOKEN_TABLE);

    crate::await_retry_or_panic!(
        token_collection.insert_one(token_db.clone(), None),
        10,
        "New token was NOT added to database".to_string(),
        &token_db,
    );
    
    Ok(())
}