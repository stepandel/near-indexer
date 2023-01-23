use tracing::info;

use bigdecimal::BigDecimal;

use crate::enums::{ActionKind, ExecutionOutcomeStatus};

use near_indexer::near_primitives::types::AccountId;
use std::collections::HashSet;

// #[derive(Insertable, Clone, Debug)]
pub struct Transaction {
    pub transaction_hash: String,
    pub included_in_block_hash: String,
    pub included_in_chunk_hash: String,
    pub index_in_chunk: i32,
    pub block_timestamp: BigDecimal,
    pub signer_account_id: String,
    pub signer_public_key: String,
    pub nonce: BigDecimal,
    pub receiver_account_id: String,
    pub signature: String,
    pub status: ExecutionOutcomeStatus,
    pub converted_into_receipt_id: String,
    pub receipt_conversion_gas_burnt: BigDecimal,
    pub receipt_conversion_tokens_burnt: BigDecimal,
}

// #[derive(Insertable, Clone, Debug)]
pub struct TransactionAction {
    pub transaction_hash: String,
    pub index_in_transaction: i32,
    pub action_kind: ActionKind,
    pub args: serde_json::Value,
}


pub(crate) fn process_transactions(transactions: &[near_indexer::IndexerTransactionWithOutcome]) {

    let nft_contract_id: AccountId = AccountId::new_unvalidated(String::from("nftsmartcontract.test.near"));
    let watched_addresses: HashSet<AccountId> = HashSet::from([nft_contract_id]);

    for transaction in transactions {

        info!(
            target: "indexer_example",
            "Transaction args: {:#?}",
            &transaction,
        );

        let transaction = &transaction.transaction;

        let receiver_id = &transaction.receiver_id;

        assert_eq!(
            watched_addresses.contains(&*receiver_id),
            true,
            "Not watching this address!"
        );

        let actions = &transaction.actions;

        let actions_iter = actions.iter();
        for action in actions_iter {
            let (_action_kind, args) =
            crate::serializers::extract_action_type_and_value_from_action_view(action);
            info!(
                target: "indexer_example",
                "Action args: {:#?}",
                &args,
            );
        }
    }
}