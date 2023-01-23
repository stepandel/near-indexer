use std::collections::HashMap;
use near_indexer::near_primitives::types::AccountId;
use crate::utils;

pub(crate) async fn mint_game_asset(
    contract_id: AccountId,
    token_id: String,
) -> anyhow::Result<()> {
    let mut url: String = crate::SERVER_BASE_URL.clone().to_owned();
    url.push_str("mintGameAsset");

    let token_db_id = utils::keccak256_hash_string(format!("{}{}", contract_id, token_id));

    let params = [("contract_id", contract_id.to_string()), ("token_id", token_id), ("near_tokend_db_id", token_db_id)];

    let args = HashMap::from(params);

    let client = reqwest::Client::new();

    crate::await_retry_or_panic!(
        client.post(url.clone()).json(&args).send(),
        10,
        "Mint request to gg-backend failed".to_string(),
        &args,
    );

    Ok(())
}

pub(crate) async fn transfer_ft(
    from_wallet_id: String,
    to_wallet_id: String,
    amount: String,
    voucher_id: Option<String>,
) -> anyhow::Result<()> {
    let mut url: String = crate::SERVER_BASE_URL.clone().to_owned();
    url.push_str("handleFungibleTokenTransfer");

    let params = [
        ("from_wallet_id", Some(from_wallet_id)),
        ("to_wallet_id", Some(to_wallet_id)),
        ("amount", Some(amount)),
        ("voucher_id", voucher_id)
    ];
    let args = HashMap::from(params);

    let client = reqwest::Client::new();

    crate::await_retry_or_panic!(
        client.post(url.clone()).json(&args).send(),
        10,
        "FT transfer request to gg-backend failed".to_string(),
        &args,
    );

    Ok(())
}